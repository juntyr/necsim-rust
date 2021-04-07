use std::{
    marker::PhantomData,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use mpi::{
    collective::{CommunicatorCollectives, Root, SystemOperation, UserOperation},
    datatype::Equivalence,
    environment::Universe,
    point_to_point::{Destination, Source},
    request::{CancelGuard, Request, StaticScope},
    topology::{Communicator, SystemCommunicator},
};

use necsim_core::{impl_report, lineage::MigratingLineage, reporter::Reporter};

use necsim_impls_no_std::{
    partitioning::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode},
    reporter::ReporterContext,
};
use necsim_impls_std::event_log::recorder::EventLogRecorder;

use crate::MpiPartitioning;

static mut MPI_LOCAL_CONTINUE: bool = false;
static mut MPI_GLOBAL_CONTINUE: bool = false;

static mut MPI_LOCAL_REMAINING: u64 = 0_u64;

static mut MPI_MIGRATION_BUFFERS: Vec<Vec<MigratingLineage>> = Vec::new();

pub struct MpiParallelPartition<P: ReporterContext> {
    _universe: Universe,
    world: SystemCommunicator,
    last_report_time: Instant,
    progress: Option<Request<'static, StaticScope>>,
    migration_buffers: Box<[Vec<MigratingLineage>]>,
    last_migration_times: Box<[Instant]>,
    emigration_requests: Box<[Option<Request<'static, StaticScope>>]>,
    barrier: Option<Request<'static, StaticScope>>,
    communicated_since_last_barrier: bool,
    recorder: EventLogRecorder,
    _marker: PhantomData<P>,
}

impl<P: ReporterContext> Drop for MpiParallelPartition<P> {
    fn drop(&mut self) {
        if let Some(progress) = self.progress.take() {
            CancelGuard::from(progress);
        }

        for request in self.emigration_requests.iter_mut() {
            if let Some(request) = request.take() {
                CancelGuard::from(request);
            }
        }

        if let Some(barrier) = self.barrier.take() {
            CancelGuard::from(barrier);
        }
    }
}

impl<P: ReporterContext> MpiParallelPartition<P> {
    const MPI_MIGRATION_WAIT_TIME: Duration = Duration::from_millis(100_u64);
    const MPI_PROGRESS_WAIT_TIME: Duration = Duration::from_millis(100_u64);

    #[must_use]
    pub fn new(universe: Universe, world: SystemCommunicator, recorder: EventLogRecorder) -> Self {
        #[allow(clippy::cast_sign_loss)]
        let world_size = world.size() as usize;

        let mut mpi_migration_buffers = Vec::with_capacity(world_size);
        mpi_migration_buffers.resize_with(world_size, Vec::new);

        unsafe { MPI_MIGRATION_BUFFERS = mpi_migration_buffers };

        let mut migration_buffers = Vec::with_capacity(world_size);
        migration_buffers.resize_with(world_size, Vec::new);

        let mut emigration_requests = Vec::with_capacity(world_size);
        emigration_requests.resize_with(world_size, || None);

        let now = Instant::now();

        Self {
            _universe: universe,
            world,
            last_report_time: now.checked_sub(Self::MPI_PROGRESS_WAIT_TIME).unwrap_or(now),
            progress: None,
            migration_buffers: migration_buffers.into_boxed_slice(),
            last_migration_times: vec![now; world_size].into_boxed_slice(),
            emigration_requests: emigration_requests.into_boxed_slice(),
            barrier: None,
            communicated_since_last_barrier: false,
            recorder,
            _marker: PhantomData::<P>,
        }
    }
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MpiParallelPartition<P> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        false
    }

    fn get_partition_rank(&self) -> u32 {
        #[allow(clippy::cast_sign_loss)]
        {
            self.world.rank() as u32
        }
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        #[allow(clippy::cast_sign_loss)]
        NonZeroU32::new(self.world.size() as u32).unwrap()
    }

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_> {
        for (partition, emigrant) in emigrants {
            self.migration_buffers[partition as usize].push(emigrant)
        }

        let self_rank_index = self.get_partition_rank() as usize;

        let now = Instant::now();

        // Receive incomming immigrating lineages
        if match immigration_mode {
            MigrationMode::Force => true,
            MigrationMode::Default => {
                now.duration_since(self.last_migration_times[self_rank_index])
                    >= Self::MPI_MIGRATION_WAIT_TIME
            },
            MigrationMode::Hold => false,
        } {
            self.last_migration_times[self_rank_index] = now;

            let immigration_buffer = &mut self.migration_buffers[self_rank_index];

            let any_process = self.world.any_process();

            // Probe MPI to receive
            while let Some((msg, status)) =
                any_process.immediate_matched_probe_with_tag(MpiPartitioning::MPI_MIGRATION_TAG)
            {
                #[allow(clippy::cast_sign_loss)]
                let number_immigrants =
                    status.count(MigratingLineage::equivalent_datatype()) as usize;

                let receive_start = immigration_buffer.len();

                immigration_buffer.reserve(number_immigrants);

                unsafe {
                    immigration_buffer.set_len(receive_start + number_immigrants);
                }

                msg.matched_receive_into(&mut immigration_buffer[receive_start..]);
            }
        }

        // Send outgoing emigrating lineages
        for rank in 0..self.get_number_of_partitions().get() {
            let rank_index = rank as usize;

            if rank_index != self_rank_index
                && match emigration_mode {
                    MigrationMode::Force => true,
                    MigrationMode::Default => {
                        now.duration_since(self.last_migration_times[rank_index])
                            >= Self::MPI_PROGRESS_WAIT_TIME
                    },
                    MigrationMode::Hold => false,
                }
            {
                // Check if the prior send request has finished
                if let Some(request) = self.emigration_requests[rank_index].take() {
                    if let Err(request) = request.test() {
                        self.emigration_requests[rank_index] = Some(request);
                    } else {
                        unsafe { MPI_MIGRATION_BUFFERS[rank_index].clear() };
                    }
                }

                let emigration_buffer = &mut self.migration_buffers[rank_index];

                // Send a new non-empty request iff the prior one has finished
                if self.emigration_requests[rank_index].is_none() && !emigration_buffer.is_empty() {
                    self.last_migration_times[rank_index] = now;

                    // MPI cannot terminate in this round since this partition gave up work
                    self.communicated_since_last_barrier = true;

                    let local_emigration_buffer: &'static mut Vec<MigratingLineage> =
                        unsafe { &mut MPI_MIGRATION_BUFFERS[rank_index] };

                    std::mem::swap(emigration_buffer, local_emigration_buffer);

                    #[allow(clippy::cast_possible_wrap)]
                    let receiver_process = self.world.process_at_rank(rank as i32);

                    self.emigration_requests[rank_index] =
                        Some(receiver_process.immediate_synchronous_send_with_tag(
                            StaticScope,
                            &local_emigration_buffer[..],
                            MpiPartitioning::MPI_MIGRATION_TAG,
                        ));
                }
            }
        }

        ImmigrantPopIterator::new(&mut self.migration_buffers[self.get_partition_rank() as usize])
    }

    fn reduce_vote_continue(&self, local_continue: bool) -> bool {
        let mut global_continue = local_continue;

        self.world.all_reduce_into(
            &local_continue,
            &mut global_continue,
            SystemOperation::logical_or(),
        );

        global_continue
    }

    fn reduce_vote_min_time(&self, local_time: f64) -> Result<f64, f64> {
        #[derive(mpi::traits::Equivalence, PartialEq, Copy, Clone)]
        struct TimePartition(f64, u32);

        let local_time_partition = TimePartition(local_time, self.get_partition_rank());
        let mut global_min_time_partition = local_time_partition;

        self.world.all_reduce_into(
            &local_time_partition,
            &mut global_min_time_partition,
            &UserOperation::commutative(|x, acc| {
                let x: &[TimePartition] = x.downcast().unwrap();
                let acc: &mut [TimePartition] = acc.downcast().unwrap();

                // Lexicographic min reduction, by time first then partition rank
                for (&x, acc) in x.iter().zip(acc) {
                    if x.0 <= acc.0 && x.1 < acc.1 {
                        *acc = x
                    }
                }
            }),
        );

        if global_min_time_partition.1 == local_time_partition.1 {
            Ok(local_time)
        } else {
            Err(global_min_time_partition.0)
        }
    }

    fn wait_for_termination(&mut self) -> bool {
        // This partition can only terminate once all migrations have been processed
        for buffer in self.migration_buffers.iter() {
            if !buffer.is_empty() {
                return true;
            }
        }
        for request in self.emigration_requests.iter() {
            if request.is_some() {
                return true;
            }
        }
        for buffer in unsafe { &mut MPI_MIGRATION_BUFFERS } {
            if !buffer.is_empty() {
                return true;
            }
        }

        // Create a new termination attempt if the last one failed
        let barrier = self.barrier.take().unwrap_or_else(|| {
            let local_continue: &'static mut bool = unsafe { &mut MPI_LOCAL_CONTINUE };
            let global_continue: &'static mut bool = unsafe { &mut MPI_GLOBAL_CONTINUE };

            *local_continue = self.communicated_since_last_barrier;
            self.communicated_since_last_barrier = false;
            *global_continue = false;

            self.world.immediate_all_reduce_into(
                StaticScope,
                local_continue,
                global_continue,
                SystemOperation::logical_or(),
            )
        });

        match barrier.test() {
            Ok(_) => {
                let global_continue: &'static mut bool = unsafe { &mut MPI_GLOBAL_CONTINUE };

                *global_continue
            },
            Err(barrier) => {
                self.barrier = Some(barrier);

                true
            },
        }
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        let mut global_time_max: f64 = 0.0_f64;
        let mut global_steps_sum: u64 = 0_u64;

        self.world
            .all_reduce_into(&local_time, &mut global_time_max, SystemOperation::max());
        self.world
            .all_reduce_into(&local_steps, &mut global_steps_sum, SystemOperation::sum());

        (global_time_max, global_steps_sum)
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

        root_process.gather_into(&remaining);
    }
}

impl<P: ReporterContext> Reporter for MpiParallelPartition<P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter
    >::ReportSpeciation> {
        event.maybe_use_in(|event| {
            self.recorder.record_speciation(event)
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter
    >::ReportDispersal> {
        event.maybe_use_in(|event| {
            self.recorder.record_dispersal(event)
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter
    >::ReportProgress> {
        remaining.maybe_use_in(|remaining| {
            if unsafe { MPI_LOCAL_REMAINING } == *remaining {
                return;
            }

            if *remaining > 0 || self.barrier.is_none() {
                let now = Instant::now();

                if now.duration_since(self.last_report_time) >= Self::MPI_PROGRESS_WAIT_TIME {
                    let progress = self.progress.take().unwrap_or_else(|| {
                        let local_remaining: &'static mut u64 = unsafe { &mut MPI_LOCAL_REMAINING };

                        self.last_report_time = now;
                        *local_remaining = *remaining;

                        let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

                        root_process.immediate_send_with_tag(
                            StaticScope,
                            local_remaining,
                            MpiPartitioning::MPI_PROGRESS_TAG,
                        )
                    });

                    if let Err(progress) = progress.test() {
                        self.progress = Some(progress);
                    }
                }
            }
        })
    });
}

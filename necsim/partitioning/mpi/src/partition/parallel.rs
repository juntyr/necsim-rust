use std::{
    fmt,
    marker::PhantomData,
    num::NonZeroU32,
    time::{Duration, Instant},
};

use mpi::{
    collective::{CommunicatorCollectives, Root, SystemOperation},
    datatype::Equivalence,
    environment::Universe,
    point_to_point::{Destination, Source},
    request::{CancelGuard, Request, StaticScope},
    topology::{Communicator, SystemCommunicator},
};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{
        boolean::{Boolean, False},
        Reporter,
    },
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode};

use crate::{
    partition::utils::{reduce_lexicographic_min_time_partition, TimePartition},
    MpiPartitioning,
};

static mut MPI_LOCAL_CONTINUE: bool = false;
static mut MPI_GLOBAL_CONTINUE: bool = false;

static mut MPI_LOCAL_REMAINING: u64 = 0_u64;

static mut MPI_MIGRATION_BUFFERS: Vec<Vec<MigratingLineage>> = Vec::new();

pub struct MpiParallelPartition<R: Reporter> {
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
    _marker: PhantomData<R>,
}

impl<R: Reporter> fmt::Debug for MpiParallelPartition<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("MpiParallelPartition").finish()
    }
}

impl<R: Reporter> Drop for MpiParallelPartition<R> {
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

impl<R: Reporter> MpiParallelPartition<R> {
    const MPI_MIGRATION_WAIT_TIME: Duration = Duration::from_millis(100_u64);
    const MPI_PROGRESS_WAIT_TIME: Duration = Duration::from_millis(100_u64);

    #[must_use]
    pub fn new(
        universe: Universe,
        world: SystemCommunicator,
        mut recorder: EventLogRecorder,
    ) -> Self {
        recorder.set_event_filter(R::ReportSpeciation::VALUE, R::ReportDispersal::VALUE);

        #[allow(clippy::cast_sign_loss)]
        let world_size = world.size() as usize;

        let mut mpi_migration_buffers = Vec::with_capacity(world_size);
        mpi_migration_buffers.resize_with(world_size, Vec::new);

        unsafe {
            MPI_MIGRATION_BUFFERS = mpi_migration_buffers;
        }

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
            _marker: PhantomData::<R>,
        }
    }
}

#[contract_trait]
impl<R: Reporter> LocalPartition<R> for MpiParallelPartition<R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type IsLive = False;
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
            self.migration_buffers[partition as usize].push(emigrant);
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
                        unsafe {
                            MPI_MIGRATION_BUFFERS[rank_index].clear();
                        }
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

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64> {
        let local_partition_rank = self.get_partition_rank();

        let global_min_time_partition = reduce_lexicographic_min_time_partition(
            self.world,
            TimePartition {
                time: local_time,
                partition: local_partition_rank,
            },
        );

        if global_min_time_partition.partition == local_partition_rank {
            Ok(local_time)
        } else {
            Err(global_min_time_partition.time)
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

    fn reduce_global_time_steps(
        &self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64) {
        let mut global_time_max = 0.0_f64;
        let mut global_steps_sum = 0_u64;

        self.world.all_reduce_into(
            &local_time.get(),
            &mut global_time_max,
            SystemOperation::max(),
        );
        self.world
            .all_reduce_into(&local_steps, &mut global_steps_sum, SystemOperation::sum());

        // Safety: `global_time_max` is the max of multiple `NonNegativeF64`s
        //         communicated through MPI
        let global_time_max = unsafe { NonNegativeF64::new_unchecked(global_time_max) };

        (global_time_max, global_steps_sum)
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

        root_process.gather_into(&remaining);
    }

    fn finalise_reporting(self) {
        std::mem::drop(self);
    }
}

impl<R: Reporter> Reporter for MpiParallelPartition<R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        self.recorder.record_speciation(speciation);
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        self.recorder.record_dispersal(dispersal);
    });

    impl_report!(progress(&mut self, remaining: MaybeUsed<R::ReportProgress>) {
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
    });
}

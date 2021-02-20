use std::{
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use mpi::{
    collective::{CommunicatorCollectives, SystemOperation},
    datatype::Equivalence,
    environment::Universe,
    point_to_point::{Destination, Source},
    request::{CancelGuard, Request, StaticScope},
    topology::{Communicator, SystemCommunicator},
};

use necsim_core::{
    event::{Event, EventType},
    lineage::MigratingLineage,
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{
    partitioning::{ImmigrantPopIterator, LocalPartition},
    reporter::{GuardedReporter, ReporterContext},
};
use necsim_impls_std::reporter::commitlog::CommitLogReporter;

use crate::MpiPartitioning;

static mut MPI_LOCAL_CONTINUE: bool = false;
static mut MPI_GLOBAL_CONTINUE: bool = false;

static mut MPI_MIGRATION_BUFFERS: Vec<Vec<MigratingLineage>> = Vec::new();

pub struct MpiRootPartition<P: ReporterContext> {
    _universe: Universe,
    world: SystemCommunicator,
    last_report_time: Instant,
    all_remaining: Box<[u64]>,
    migration_buffers: Box<[Vec<MigratingLineage>]>,
    last_migration_times: Box<[Instant]>,
    emigration_requests: Box<[Option<Request<'static, StaticScope>>]>,
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
    event_reporter: CommitLogReporter,
    barrier: Option<Request<'static, StaticScope>>,
    communicated_since_last_barrier: bool,
}

impl<P: ReporterContext> Drop for MpiRootPartition<P> {
    fn drop(&mut self) {
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

impl<P: ReporterContext> MpiRootPartition<P> {
    const MPI_MIGRATION_WAIT_TIME: Duration = Duration::from_millis(100_u64);
    const MPI_PROGRESS_WAIT_TIME: Duration = Duration::from_millis(100_u64);

    #[must_use]
    pub fn new(
        universe: Universe,
        world: SystemCommunicator,
        reporter: GuardedReporter<P::Reporter, P::Finaliser>,
        event_log_path: &Path,
    ) -> Self {
        let mut event_log_path = PathBuf::from(event_log_path);
        event_log_path.push(world.rank().to_string());

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
            all_remaining: vec![0; world_size].into_boxed_slice(),
            migration_buffers: migration_buffers.into_boxed_slice(),
            last_migration_times: vec![now; world_size].into_boxed_slice(),
            emigration_requests: emigration_requests.into_boxed_slice(),
            reporter,
            event_reporter: CommitLogReporter::try_new(&event_log_path).unwrap(),
            barrier: None,
            communicated_since_last_barrier: false,
        }
    }
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MpiRootPartition<P> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        true
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
    ) -> Self::ImmigrantIterator<'_> {
        for (partition, emigrant) in emigrants {
            self.migration_buffers[partition as usize].push(emigrant)
        }

        let self_rank_index = self.get_partition_rank() as usize;

        let now = Instant::now();

        // Receive incomming immigrating lineages
        if now.duration_since(self.last_migration_times[self_rank_index])
            >= Self::MPI_MIGRATION_WAIT_TIME
        {
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
                && now.duration_since(self.last_migration_times[rank_index])
                    >= Self::MPI_PROGRESS_WAIT_TIME
            {
                // Check if the prior send request has finished
                if let Some(request) = self.emigration_requests[rank_index].take() {
                    if let Err(request) = request.test() {
                        self.emigration_requests[rank_index] = Some(request);
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
                    emigration_buffer.clear();

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

                if !*global_continue {
                    self.reporter.report_progress(0);
                }

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
}

impl<P: ReporterContext> Reporter for MpiRootPartition<P> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            self.event_reporter.report_event(event);
        }
    }

    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        let now = Instant::now();

        if now.duration_since(self.last_report_time) >= Self::MPI_PROGRESS_WAIT_TIME {
            self.last_report_time = now;

            self.all_remaining[MpiPartitioning::ROOT_RANK as usize] = remaining;

            let any_process = self.world.any_process();

            while let Some((msg, _)) =
                any_process.immediate_matched_probe_with_tag(MpiPartitioning::MPI_PROGRESS_TAG)
            {
                let remaining_status: (u64, _) = msg.matched_receive();

                #[allow(clippy::cast_sign_loss)]
                self.all_remaining[remaining_status.1.source_rank() as usize] = remaining_status.0;
            }

            self.reporter
                .report_progress(self.all_remaining.iter().sum());
        }
    }
}

impl<P: ReporterContext> EventFilter for MpiRootPartition<P> {
    const REPORT_DISPERSAL: bool = P::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::Reporter::REPORT_SPECIATION;
}

use std::{
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::Instant,
};

use mpi::{
    collective::{CommunicatorCollectives, SystemOperation},
    environment::Universe,
    point_to_point::Source,
    request::{CancelGuard, Request, StaticScope},
    topology::{Communicator, SystemCommunicator},
    Tag,
};

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{
    partitioning::LocalPartition,
    reporter::{GuardedReporter, ReporterContext},
};
use necsim_impls_std::reporter::commitlog::CommitLogReporter;

use crate::MpiPartitioning;

static mut MPI_LOCAL_CONTINUE: bool = false;
static mut MPI_GLOBAL_CONTINUE: bool = false;

pub struct MpiRootPartition<P: ReporterContext> {
    _universe: Universe,
    world: SystemCommunicator,
    last_mpi_call_time: Instant,
    all_remaining: Box<[u64]>,
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
    event_reporter: CommitLogReporter,
    barrier: Option<Request<'static, StaticScope>>,
    communicated_since_last_barrier: bool,
}

impl<P: ReporterContext> Drop for MpiRootPartition<P> {
    fn drop(&mut self) {
        if let Some(barrier) = self.barrier.take() {
            CancelGuard::from(barrier);
        }
    }
}

impl<P: ReporterContext> MpiRootPartition<P> {
    pub(super) const MPI_PROGRESS_TAG: Tag = 0;
    const MPI_WAIT_TIME: f64 = 0.01_f64;

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

        Self {
            _universe: universe,
            world,
            last_mpi_call_time: Instant::now(),
            all_remaining: vec![0; world_size].into_boxed_slice(),
            reporter,
            event_reporter: CommitLogReporter::try_new(&event_log_path).unwrap(),
            barrier: None,
            communicated_since_last_barrier: false,
        }
    }
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MpiRootPartition<P> {
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

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        let mut global_time_max: f64 = 0.0_f64;
        let mut global_steps_sum: u64 = 0_u64;

        self.world
            .all_reduce_into(&local_time, &mut global_time_max, SystemOperation::max());
        self.world
            .all_reduce_into(&local_steps, &mut global_steps_sum, SystemOperation::sum());

        (global_time_max, global_steps_sum)
    }

    fn wait_for_termination(&mut self) -> bool {
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

        if now.duration_since(self.last_mpi_call_time).as_secs_f64() >= Self::MPI_WAIT_TIME {
            self.last_mpi_call_time = now;

            self.all_remaining[MpiPartitioning::ROOT_RANK as usize] = remaining;

            let any_process = self.world.any_process();

            while let Some((msg, _)) =
                any_process.immediate_matched_probe_with_tag(Self::MPI_PROGRESS_TAG)
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

use std::{
    marker::PhantomData,
    num::NonZeroU32,
    path::{Path, PathBuf},
    time::Instant,
};

use mpi::{
    collective::{CommunicatorCollectives, SystemOperation},
    environment::Universe,
    point_to_point::Destination,
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
    reporter::ReporterContext,
};
use necsim_impls_std::reporter::commitlog::CommitLogReporter;

use crate::{partition::root::MpiRootPartition, MpiPartitioning};

static mut MPI_LOCAL_CONTINUE: bool = false;
static mut MPI_GLOBAL_CONTINUE: bool = false;

static mut LOCAL_REMAINING: u64 = 0_u64;

pub struct MpiParallelPartition<P: ReporterContext> {
    _universe: Universe,
    world: SystemCommunicator,
    last_mpi_call_time: Instant,
    progress: Option<Request<'static, StaticScope>>,
    barrier: Option<Request<'static, StaticScope>>,
    communicated_since_last_barrier: bool,
    event_reporter: CommitLogReporter,
    _marker: PhantomData<P>,
}

impl<P: ReporterContext> Drop for MpiParallelPartition<P> {
    fn drop(&mut self) {
        if let Some(progress) = self.progress.take() {
            CancelGuard::from(progress);
        }

        if let Some(barrier) = self.barrier.take() {
            CancelGuard::from(barrier);
        }
    }
}

impl<P: ReporterContext> MpiParallelPartition<P> {
    const MPI_WAIT_TIME: f64 = 0.1_f64;

    #[must_use]
    pub fn new(universe: Universe, world: SystemCommunicator, event_log_path: &Path) -> Self {
        let mut event_log_path = PathBuf::from(event_log_path);
        event_log_path.push(world.rank().to_string());

        Self {
            _universe: universe,
            world,
            last_mpi_call_time: Instant::now(),
            progress: None,
            barrier: None,
            communicated_since_last_barrier: false,
            event_reporter: CommitLogReporter::try_new(&event_log_path).unwrap(),
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
        _emigrants: &mut E,
    ) -> Self::ImmigrantIterator<'_> {
        // TODO: call `self.event_reporter.mark_disjoint()` on any individual
        //       exchange call (i.e. send or test for receive of migration)

        unimplemented!("TODO: migrate_individuals from MpiParallelPartition")
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

impl<P: ReporterContext> Reporter for MpiParallelPartition<P> {
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
        if unsafe { LOCAL_REMAINING } == remaining {
            return;
        }

        if remaining > 0 || self.barrier.is_none() {
            let now = Instant::now();

            if now.duration_since(self.last_mpi_call_time).as_secs_f64() >= Self::MPI_WAIT_TIME {
                let progress = self.progress.take().unwrap_or_else(|| {
                    let local_remaining: &'static mut u64 = unsafe { &mut LOCAL_REMAINING };

                    self.last_mpi_call_time = now;
                    *local_remaining = remaining;

                    let root_process = self.world.process_at_rank(MpiPartitioning::ROOT_RANK);

                    root_process.immediate_send_with_tag(
                        StaticScope,
                        local_remaining,
                        MpiRootPartition::<P>::MPI_PROGRESS_TAG,
                    )
                });

                if let Err(progress) = progress.test() {
                    self.progress = Some(progress);
                }
            }
        }
    }
}

impl<P: ReporterContext> EventFilter for MpiParallelPartition<P> {
    const REPORT_DISPERSAL: bool = P::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::Reporter::REPORT_SPECIATION;
}

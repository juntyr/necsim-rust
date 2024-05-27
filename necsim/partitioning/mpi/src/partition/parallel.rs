use std::{
    fmt,
    marker::PhantomData,
    ops::ControlFlow,
    time::{Duration, Instant},
};

use mpi::{
    collective::Root, environment::Universe, point_to_point::Destination, topology::Communicator,
};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{
        boolean::{Boolean, False},
        Reporter,
    },
};
use necsim_core_bond::PositiveF64;

use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{
    iterator::ImmigrantPopIterator, partition::Partition, LocalPartition, MigrationMode,
};

use crate::{
    partition::{common::MpiCommonPartition, utils::MpiMigratingLineage},
    request::DataOrRequest,
    MpiPartitioning,
};

pub struct MpiParallelPartition<'p, R: Reporter> {
    common: MpiCommonPartition<'p>,
    mpi_local_remaining: DataOrRequest<'p, u64, u64>,
    last_report_time: Instant,
    recorder: EventLogRecorder,
    progress_interval: Duration,
    _marker: PhantomData<(&'p (), R)>,
}

impl<'p, R: Reporter> fmt::Debug for MpiParallelPartition<'p, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(MpiParallelPartition)).finish()
    }
}

impl<'p, R: Reporter> MpiParallelPartition<'p, R> {
    #[must_use]
    pub(crate) fn new(
        universe: Universe,
        mpi_local_global_wait: DataOrRequest<'p, (bool, bool), bool>,
        mpi_local_remaining: DataOrRequest<'p, u64, u64>,
        mpi_emigration_buffers: Box<
            [DataOrRequest<'p, Vec<MigratingLineage>, [MpiMigratingLineage]>],
        >,
        mut recorder: EventLogRecorder,
        migration_interval: Duration,
        progress_interval: Duration,
    ) -> Self {
        recorder.set_event_filter(R::ReportSpeciation::VALUE, R::ReportDispersal::VALUE);

        let now = Instant::now();

        let common = MpiCommonPartition::new(
            universe,
            mpi_local_global_wait,
            mpi_emigration_buffers,
            now,
            migration_interval,
        );

        Self {
            common,
            mpi_local_remaining,
            last_report_time: now.checked_sub(progress_interval).unwrap_or(now),
            recorder,
            progress_interval,
            _marker: PhantomData::<(&'p (), R)>,
        }
    }
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for MpiParallelPartition<'p, R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a> where 'p: 'a, R: 'a;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        false
    }

    fn get_partition(&self) -> Partition {
        self.common.get_partition()
    }

    fn migrate_individuals<'a, E: Iterator<Item = (u32, MigratingLineage)>>(
        &'a mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'a>
    where
        'p: 'a,
    {
        self.common
            .migrate_individuals(emigrants, emigration_mode, immigration_mode)
    }

    fn reduce_vote_any(&mut self, vote: bool) -> bool {
        self.common.reduce_vote_any(vote)
    }

    fn reduce_vote_min_time(
        &mut self,
        local_time: PositiveF64,
    ) -> Result<PositiveF64, PositiveF64> {
        self.common.reduce_vote_min_time(local_time)
    }

    fn wait_for_termination(&mut self) -> ControlFlow<(), ()> {
        self.common.wait_for_termination()
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        let root_process = self
            .common
            .world()
            .process_at_rank(MpiPartitioning::ROOT_RANK);

        root_process.gather_into(&remaining);
    }

    fn finalise_reporting(self) {
        std::mem::drop(self);
    }
}

impl<'p, R: Reporter> Reporter for MpiParallelPartition<'p, R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        self.recorder.record_speciation(speciation);
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        self.recorder.record_dispersal(dispersal);
    });

    impl_report!(progress(&mut self, remaining: MaybeUsed<R::ReportProgress>) {
        if self.mpi_local_remaining.test_for_data_mut().map_or(false, |local_remaining| *local_remaining == *remaining) {
            return;
        }

        // Only send progress if there is no ongoing continue barrier request
        if !self.common.has_ongoing_termination_vote() {
            let now = Instant::now();

            if now.duration_since(self.last_report_time) >= self.progress_interval {
                let root_process = self.common.world().process_at_rank(MpiPartitioning::ROOT_RANK);

                let mut last_report_time = self.last_report_time;

                self.mpi_local_remaining.request_if_data(|local_remaining, scope| {
                    last_report_time = now;

                    *local_remaining = *remaining;

                    root_process.immediate_send_with_tag(
                        scope,
                        local_remaining,
                        MpiPartitioning::MPI_PROGRESS_TAG,
                    )
                });

                self.last_report_time = last_report_time;
            }
        }
    });
}

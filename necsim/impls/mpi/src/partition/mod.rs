use std::num::NonZeroU32;

use necsim_core::{impl_report, lineage::MigratingLineage, reporter::Reporter};

use necsim_impls_no_std::{
    partitioning::{
        iterator::ImmigrantPopIterator, monolithic::live::LiveMonolithicLocalPartition,
        LocalPartition, MigrationMode,
    },
    reporter::ReporterContext,
};

use necsim_impls_std::partitioning::monolithic::recorded::RecordedMonolithicLocalPartition;

mod parallel;
mod root;

pub use parallel::MpiParallelPartition;
pub use root::MpiRootPartition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum MpiLocalPartition<P: ReporterContext> {
    LiveMonolithic(Box<LiveMonolithicLocalPartition<P>>),
    RecordedMonolithic(Box<RecordedMonolithicLocalPartition<P>>),
    Root(Box<MpiRootPartition<P>>),
    Parallel(Box<MpiParallelPartition<P>>),
}

#[contract_trait]
impl<P: ReporterContext> LocalPartition<P> for MpiLocalPartition<P> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        match self {
            Self::LiveMonolithic(partition) => partition.is_root(),
            Self::RecordedMonolithic(partition) => partition.is_root(),
            Self::Root(partition) => partition.is_root(),
            Self::Parallel(partition) => partition.is_root(),
        }
    }

    fn get_partition_rank(&self) -> u32 {
        match self {
            Self::LiveMonolithic(partition) => partition.get_partition_rank(),
            Self::RecordedMonolithic(partition) => partition.get_partition_rank(),
            Self::Root(partition) => partition.get_partition_rank(),
            Self::Parallel(partition) => partition.get_partition_rank(),
        }
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        match self {
            Self::LiveMonolithic(partition) => partition.get_number_of_partitions(),
            Self::RecordedMonolithic(partition) => partition.get_number_of_partitions(),
            Self::Root(partition) => partition.get_number_of_partitions(),
            Self::Parallel(partition) => partition.get_number_of_partitions(),
        }
    }

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        emigration_mode: MigrationMode,
        immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_> {
        match self {
            Self::LiveMonolithic(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
            Self::RecordedMonolithic(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
            Self::Root(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
            Self::Parallel(partition) => {
                partition.migrate_individuals(emigrants, emigration_mode, immigration_mode)
            },
        }
    }

    fn reduce_vote_continue(&self, local_continue: bool) -> bool {
        match self {
            Self::LiveMonolithic(partition) => partition.reduce_vote_continue(local_continue),
            Self::RecordedMonolithic(partition) => partition.reduce_vote_continue(local_continue),
            Self::Root(partition) => partition.reduce_vote_continue(local_continue),
            Self::Parallel(partition) => partition.reduce_vote_continue(local_continue),
        }
    }

    fn reduce_vote_min_time(&self, local_time: f64) -> Result<f64, f64> {
        match self {
            Self::LiveMonolithic(partition) => partition.reduce_vote_min_time(local_time),
            Self::RecordedMonolithic(partition) => partition.reduce_vote_min_time(local_time),
            Self::Root(partition) => partition.reduce_vote_min_time(local_time),
            Self::Parallel(partition) => partition.reduce_vote_min_time(local_time),
        }
    }

    fn wait_for_termination(&mut self) -> bool {
        match self {
            Self::LiveMonolithic(partition) => partition.wait_for_termination(),
            Self::RecordedMonolithic(partition) => partition.wait_for_termination(),
            Self::Root(partition) => partition.wait_for_termination(),
            Self::Parallel(partition) => partition.wait_for_termination(),
        }
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        match self {
            Self::LiveMonolithic(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
            Self::RecordedMonolithic(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
            Self::Root(partition) => partition.reduce_global_time_steps(local_time, local_steps),
            Self::Parallel(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
        }
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        match self {
            Self::LiveMonolithic(partition) => partition.report_progress_sync(remaining),
            Self::RecordedMonolithic(partition) => partition.report_progress_sync(remaining),
            Self::Root(partition) => partition.report_progress_sync(remaining),
            Self::Parallel(partition) => partition.report_progress_sync(remaining),
        }
    }
}

impl<P: ReporterContext> Reporter for MpiLocalPartition<P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter
    >::ReportSpeciation> {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_speciation(event),
            Self::RecordedMonolithic(partition) => partition.get_reporter().report_speciation(event),
            Self::Root(partition) => partition.get_reporter().report_speciation(event),
            Self::Parallel(partition) => partition.get_reporter().report_speciation(event),
        }
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter>::ReportDispersal
    > {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_dispersal(event),
            Self::RecordedMonolithic(partition) => partition.get_reporter().report_dispersal(event),
            Self::Root(partition) => partition.get_reporter().report_dispersal(event),
            Self::Parallel(partition) => partition.get_reporter().report_dispersal(event),
        }
    });

    impl_report!(progress(&mut self, remaining: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter
    >::ReportProgress> {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_progress(remaining),
            Self::RecordedMonolithic(partition) => {
                partition.get_reporter().report_progress(remaining)
            },
            Self::Root(partition) => partition.get_reporter().report_progress(remaining),
            Self::Parallel(partition) => partition.get_reporter().report_progress(remaining),
        }
    });
}

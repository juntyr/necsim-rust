use std::num::NonZeroU32;

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{boolean::True, Reporter},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_partitioning_core::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode};
use necsim_partitioning_monolithic::{
    live::LiveMonolithicLocalPartition, recorded::RecordedMonolithicLocalPartition,
};

mod parallel;
mod root;
mod utils;

#[allow(clippy::module_name_repetitions)]
pub use parallel::MpiParallelPartition;
#[allow(clippy::module_name_repetitions)]
pub use root::MpiRootPartition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum MpiLocalPartition<R: Reporter> {
    LiveMonolithic(Box<LiveMonolithicLocalPartition<R>>),
    RecordedMonolithic(Box<RecordedMonolithicLocalPartition<R>>),
    Root(Box<MpiRootPartition<R>>),
    Parallel(Box<MpiParallelPartition<R>>),
}

#[contract_trait]
impl<R: Reporter> LocalPartition<R> for MpiLocalPartition<R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type IsLive = True;
    // pessimistic
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

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64> {
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

    fn reduce_global_time_steps(
        &self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64) {
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

    fn finalise_reporting(self) {
        match self {
            Self::LiveMonolithic(partition) => partition.finalise_reporting(),
            Self::RecordedMonolithic(partition) => partition.finalise_reporting(),
            Self::Root(partition) => partition.finalise_reporting(),
            Self::Parallel(partition) => partition.finalise_reporting(),
        }
    }
}

impl<R: Reporter> Reporter for MpiLocalPartition<R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
            Self::RecordedMonolithic(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
            Self::Root(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
            Self::Parallel(partition) => partition.get_reporter().report_speciation(
                speciation.into()
            ),
        }
    });

    impl_report!(dispersal(&mut self, dispersal: MaybeUsed<R::ReportDispersal>) {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
            Self::RecordedMonolithic(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
            Self::Root(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
            Self::Parallel(partition) => partition.get_reporter().report_dispersal(
                dispersal.into()
            ),
        }
    });

    impl_report!(progress(&mut self, progress: MaybeUsed<R::ReportProgress>) {
        match self {
            Self::LiveMonolithic(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
            Self::RecordedMonolithic(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
            Self::Root(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
            Self::Parallel(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
        }
    });
}

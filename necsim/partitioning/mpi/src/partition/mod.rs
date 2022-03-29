use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{boolean::False, Reporter},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_partitioning_core::{
    iterator::ImmigrantPopIterator, partition::Partition, LocalPartition, MigrationMode,
};

mod parallel;
mod root;
mod utils;

#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use parallel::MpiParallelPartition;
#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use root::MpiRootPartition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub enum MpiLocalPartition<'p, R: Reporter> {
    Root(Box<MpiRootPartition<'p, R>>),
    Parallel(Box<MpiParallelPartition<'p, R>>),
}

#[contract_trait]
impl<'p, R: Reporter> LocalPartition<'p, R> for MpiLocalPartition<'p, R> {
    type ImmigrantIterator<'a>:
    where
        'p: 'a,
        R: 'a,
    = ImmigrantPopIterator<'a>;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        match self {
            Self::Root(partition) => partition.is_root(),
            Self::Parallel(partition) => partition.is_root(),
        }
    }

    fn get_partition(&self) -> Partition {
        match self {
            Self::Root(partition) => partition.get_partition(),
            Self::Parallel(partition) => partition.get_partition(),
        }
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
        match self {
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
            Self::Root(partition) => partition.reduce_vote_continue(local_continue),
            Self::Parallel(partition) => partition.reduce_vote_continue(local_continue),
        }
    }

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64> {
        match self {
            Self::Root(partition) => partition.reduce_vote_min_time(local_time),
            Self::Parallel(partition) => partition.reduce_vote_min_time(local_time),
        }
    }

    fn wait_for_termination(&mut self) -> bool {
        match self {
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
            Self::Root(partition) => partition.reduce_global_time_steps(local_time, local_steps),
            Self::Parallel(partition) => {
                partition.reduce_global_time_steps(local_time, local_steps)
            },
        }
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        match self {
            Self::Root(partition) => partition.report_progress_sync(remaining),
            Self::Parallel(partition) => partition.report_progress_sync(remaining),
        }
    }

    fn finalise_reporting(self) {
        match self {
            Self::Root(partition) => partition.finalise_reporting(),
            Self::Parallel(partition) => partition.finalise_reporting(),
        }
    }
}

impl<'p, R: Reporter> Reporter for MpiLocalPartition<'p, R> {
    impl_report!(speciation(&mut self, speciation: MaybeUsed<R::ReportSpeciation>) {
        match self {
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
            Self::Root(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
            Self::Parallel(partition) => partition.get_reporter().report_progress(
                progress.into()
            ),
        }
    });
}

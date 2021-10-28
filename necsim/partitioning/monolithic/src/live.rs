use std::fmt;

use necsim_core::{
    lineage::MigratingLineage,
    reporter::{boolean::True, FilteredReporter, Reporter},
};
use necsim_core_bond::{NonNegativeF64, Partition, PositiveF64};

use necsim_partitioning_core::{
    context::ReporterContext, iterator::ImmigrantPopIterator, LocalPartition, MigrationMode,
};

#[allow(clippy::module_name_repetitions)]
pub struct LiveMonolithicLocalPartition<R: Reporter> {
    reporter: FilteredReporter<R, True, True, True>,
    loopback: Vec<MigratingLineage>,
}

impl<R: Reporter> fmt::Debug for LiveMonolithicLocalPartition<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct LoopbackLen(usize);

        impl fmt::Debug for LoopbackLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<MigratingLineage; {}>", self.0)
            }
        }

        fmt.debug_struct(stringify!(LiveMonolithicLocalPartition))
            .field("reporter", &self.reporter)
            .field("loopback", &LoopbackLen(self.loopback.len()))
            .finish()
    }
}

#[contract_trait]
impl<R: Reporter> LocalPartition<R> for LiveMonolithicLocalPartition<R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type IsLive = True;
    type Reporter = FilteredReporter<R, True, True, True>;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        &mut self.reporter
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_partition(&self) -> Partition {
        Partition::monolithic()
    }

    fn migrate_individuals<E: Iterator<Item = (u32, MigratingLineage)>>(
        &mut self,
        emigrants: &mut E,
        _emigration_mode: MigrationMode,
        _immigration_mode: MigrationMode,
    ) -> Self::ImmigrantIterator<'_> {
        for (_, emigrant) in emigrants {
            self.loopback.push(emigrant);
        }

        ImmigrantPopIterator::new(&mut self.loopback)
    }

    fn reduce_vote_continue(&self, local_continue: bool) -> bool {
        local_continue
    }

    fn reduce_vote_min_time(&self, local_time: PositiveF64) -> Result<PositiveF64, PositiveF64> {
        Ok(local_time)
    }

    fn wait_for_termination(&mut self) -> bool {
        !self.loopback.is_empty()
    }

    fn reduce_global_time_steps(
        &self,
        local_time: NonNegativeF64,
        local_steps: u64,
    ) -> (NonNegativeF64, u64) {
        (local_time, local_steps)
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        self.reporter.report_progress(&remaining.into());
    }

    fn finalise_reporting(self) {
        self.reporter.finalise();
    }
}

impl<R: Reporter> LiveMonolithicLocalPartition<R> {
    pub fn from_reporter(reporter: FilteredReporter<R, True, True, True>) -> Self {
        Self {
            reporter,
            loopback: Vec::new(),
        }
    }

    /// # Errors
    ///
    /// Returns any error which occured while building the context's reporter
    pub fn try_from_context<P: ReporterContext<Reporter = R>>(context: P) -> anyhow::Result<Self> {
        Ok(Self::from_reporter(context.try_build()?))
    }
}

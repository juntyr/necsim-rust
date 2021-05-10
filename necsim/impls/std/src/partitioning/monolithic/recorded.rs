use std::{fmt, num::NonZeroU32};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{
        boolean::{Boolean, False, True},
        used::Unused,
        FilteredReporter, Reporter,
    },
};

use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_impls_no_std::{
    partitioning::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode},
    reporter::ReporterContext,
};

use crate::event_log::recorder::EventLogRecorder;

use anyhow::Result;

#[allow(clippy::module_name_repetitions)]
pub struct RecordedMonolithicLocalPartition<R: Reporter> {
    reporter: FilteredReporter<R, False, False, True>,
    recorder: EventLogRecorder,
    loopback: Vec<MigratingLineage>,
}

impl<R: Reporter> fmt::Debug for RecordedMonolithicLocalPartition<R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct LoopbackLen(usize);

        impl fmt::Debug for LoopbackLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<MigratingLineage; {}>", self.0)
            }
        }

        fmt.debug_struct("RecordedMonolithicLocalPartition")
            .field("reporter", &self.reporter)
            .field("recorder", &self.recorder)
            .field("loopback", &LoopbackLen(self.loopback.len()))
            .finish()
    }
}

#[contract_trait]
impl<R: Reporter> LocalPartition<R> for RecordedMonolithicLocalPartition<R> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
    type IsLive = False;
    type Reporter = Self;

    fn get_reporter(&mut self) -> &mut Self::Reporter {
        self
    }

    fn is_root(&self) -> bool {
        true
    }

    fn get_partition_rank(&self) -> u32 {
        0
    }

    fn get_number_of_partitions(&self) -> NonZeroU32 {
        unsafe { NonZeroU32::new_unchecked(1) }
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
        self.reporter.report_progress(Unused::new(&remaining));
    }

    fn finalise_reporting(self) {
        self.reporter.finalise()
    }
}

impl<R: Reporter> RecordedMonolithicLocalPartition<R> {
    /// # Errors
    ///
    /// Returns any error which occured while building the context's reporter
    pub fn try_from_context_and_recorder<P: ReporterContext<Reporter = R>>(
        context: P,
        mut recorder: EventLogRecorder,
    ) -> anyhow::Result<Self> {
        recorder.set_event_filter(R::ReportSpeciation::VALUE, R::ReportDispersal::VALUE);

        Ok(Self {
            reporter: context.try_build()?,
            recorder,
            loopback: Vec::new(),
        })
    }
}

impl<R: Reporter> Reporter for RecordedMonolithicLocalPartition<R> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<R::ReportSpeciation> {
        event.maybe_use_in(|event| {
            self.recorder.record_speciation(event)
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<R::ReportDispersal> {
        event.maybe_use_in(|event| {
            self.recorder.record_dispersal(event)
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> MaybeUsed<R::ReportProgress> {
        self.reporter.report_progress(remaining)
    });
}

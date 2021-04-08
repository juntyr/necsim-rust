use std::{fmt, num::NonZeroU32};

use necsim_core::{
    impl_report,
    lineage::MigratingLineage,
    reporter::{used::Unused, Reporter},
};

use necsim_impls_no_std::{
    partitioning::{iterator::ImmigrantPopIterator, LocalPartition, MigrationMode},
    reporter::{GuardedReporter, ReporterContext},
};

use crate::event_log::recorder::EventLogRecorder;

use anyhow::Result;

#[allow(clippy::module_name_repetitions)]
pub struct RecordedMonolithicLocalPartition<P: ReporterContext> {
    reporter: GuardedReporter<P::Reporter, P::Finaliser>,
    recorder: EventLogRecorder,
    loopback: Vec<MigratingLineage>,
}

impl<P: ReporterContext> fmt::Debug for RecordedMonolithicLocalPartition<P> {
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        struct LoopbackLen(usize);

        impl fmt::Debug for LoopbackLen {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
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
impl<P: ReporterContext> LocalPartition<P> for RecordedMonolithicLocalPartition<P> {
    type ImmigrantIterator<'a> = ImmigrantPopIterator<'a>;
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

    fn reduce_vote_min_time(&self, local_time: f64) -> Result<f64, f64> {
        Ok(local_time)
    }

    fn wait_for_termination(&mut self) -> bool {
        !self.loopback.is_empty()
    }

    fn reduce_global_time_steps(&self, local_time: f64, local_steps: u64) -> (f64, u64) {
        (local_time, local_steps)
    }

    fn report_progress_sync(&mut self, remaining: u64) {
        self.reporter.report_progress(Unused::new(&remaining));
    }
}

impl<P: ReporterContext> RecordedMonolithicLocalPartition<P> {
    pub fn from_reporter_and_recorder(
        reporter_guard: GuardedReporter<P::Reporter, P::Finaliser>,
        recorder: EventLogRecorder,
    ) -> Self {
        Self {
            reporter: reporter_guard,
            recorder,
            loopback: Vec::new(),
        }
    }
}

impl<P: ReporterContext> Reporter for RecordedMonolithicLocalPartition<P> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
            <<P as ReporterContext>::Reporter as Reporter>::ReportSpeciation
    > {
        event.maybe_use_in(|event| {
            self.recorder.record_speciation(event)
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter>::ReportDispersal
    > {
        event.maybe_use_in(|event| {
            self.recorder.record_dispersal(event)
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> MaybeUsed<
        <<P as ReporterContext>::Reporter as Reporter>::ReportProgress
    > {
        self.reporter.report_progress(remaining)
    });
}

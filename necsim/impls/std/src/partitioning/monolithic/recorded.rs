use std::num::NonZeroU32;

use necsim_core::{
    event::{Event, EventType},
    lineage::MigratingLineage,
    reporter::{EventFilter, Reporter},
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
        self.reporter.report_progress(remaining)
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
    #[inline]
    fn report_event(&mut self, event: &Event) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            self.recorder.record_event(event);
        }
    }

    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        self.reporter.report_progress(remaining)
    }
}

impl<P: ReporterContext> EventFilter for RecordedMonolithicLocalPartition<P> {
    const REPORT_DISPERSAL: bool = P::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::Reporter::REPORT_SPECIATION;
}

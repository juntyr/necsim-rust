use std::marker::PhantomData;

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

pub struct PartitionReporterProxy<'p, R: ReporterContext, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    _marker: PhantomData<R>,
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> EventFilter
    for PartitionReporterProxy<'p, R, P>
{
    const REPORT_DISPERSAL: bool = R::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = R::Reporter::REPORT_SPECIATION;
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> Reporter for PartitionReporterProxy<'p, R, P> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            self.local_partition.get_reporter().report_event(event)
        }
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // Ignore the progress from the individual simulations
    }
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> PartitionReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            _marker: PhantomData::<R>,
        }
    }

    #[inline]
    pub fn report_total_progress(&mut self, remaining: u64) {
        self.local_partition
            .get_reporter()
            .report_progress(remaining)
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}

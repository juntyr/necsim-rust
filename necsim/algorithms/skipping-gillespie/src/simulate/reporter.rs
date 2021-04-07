use std::marker::PhantomData;

use necsim_core::{
    event::{EventType, PackedEvent},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

pub struct PartitionReporterProxy<'p, R: ReporterContext, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    event_buffer: Vec<PackedEvent>,
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
    fn report_event(&mut self, event: &PackedEvent) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            self.event_buffer.push(event.clone())
        }
    }

    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        self.local_partition
            .get_reporter()
            .report_progress(remaining)
    }
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> PartitionReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            event_buffer: Vec::new(),
            _marker: PhantomData::<R>,
        }
    }

    pub fn clear_events(&mut self) {
        self.event_buffer.clear()
    }

    pub fn report_events(&mut self) {
        for event in &self.event_buffer {
            self.local_partition.get_reporter().report_event(event)
        }

        self.clear_events()
    }

    pub fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}

use std::marker::PhantomData;

use hashbrown::{hash_map::RawEntryMut, HashMap};

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

pub struct DeduplicatingReporterProxy<'p, R: ReporterContext, P: LocalPartition<R>> {
    local_partition: &'p mut P,
    event_deduplicator: HashMap<Event, ()>,
    _marker: PhantomData<R>,
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> EventFilter
    for DeduplicatingReporterProxy<'p, R, P>
{
    const REPORT_DISPERSAL: bool = R::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = R::Reporter::REPORT_SPECIATION;
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> Reporter
    for DeduplicatingReporterProxy<'p, R, P>
{
    #[inline]
    fn report_event(&mut self, event: &Event) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            if let RawEntryMut::Vacant(entry) =
                self.event_deduplicator.raw_entry_mut().from_key(event)
            {
                self.local_partition
                    .get_reporter()
                    .report_event(entry.insert(event.clone(), ()).0)
            }
        }
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // Ignore the progress from the individual simulations
    }
}

impl<'p, R: ReporterContext, P: LocalPartition<R>> DeduplicatingReporterProxy<'p, R, P> {
    pub fn from(local_partition: &'p mut P) -> Self {
        Self {
            local_partition,
            event_deduplicator: HashMap::new(),
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

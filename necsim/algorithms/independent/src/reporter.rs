use hashbrown::{hash_map::RawEntryMut, HashMap};

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

pub struct DeduplicatingReporterProxy<'r, P: Reporter> {
    reporter: &'r mut P,
    event_deduplicator: HashMap<Event, ()>,
}

impl<'r, P: Reporter> EventFilter for DeduplicatingReporterProxy<'r, P> {
    const REPORT_DISPERSAL: bool = P::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = P::REPORT_SPECIATION;
}

impl<'r, P: Reporter> Reporter for DeduplicatingReporterProxy<'r, P> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            if let RawEntryMut::Vacant(entry) =
                self.event_deduplicator.raw_entry_mut().from_key(event)
            {
                self.reporter
                    .report_event(entry.insert(event.clone(), ()).0)
            }
        }
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // Ignore the progress from the individual simulations
    }
}

impl<'r, P: Reporter> DeduplicatingReporterProxy<'r, P> {
    pub fn from(reporter: &'r mut P) -> Self {
        Self {
            reporter,
            event_deduplicator: HashMap::new(),
        }
    }

    #[inline]
    pub fn report_total_progress(&mut self, remaining: u64) {
        self.reporter.report_progress(remaining)
    }
}

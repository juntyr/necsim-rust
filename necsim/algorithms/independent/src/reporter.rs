use hashbrown::{hash_map::RawEntryMut, HashMap};

use necsim_core::{
    event::Event,
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
    fn report_event(&mut self, event: &Event) {
        if let RawEntryMut::Vacant(entry) = self.event_deduplicator.raw_entry_mut().from_key(event)
        {
            self.reporter
                .report_event(entry.insert(event.clone(), ()).0)
        }
    }
}

impl<'r, P: Reporter> DeduplicatingReporterProxy<'r, P> {
    pub fn from(reporter: &'r mut P) -> Self {
        Self {
            reporter,
            event_deduplicator: HashMap::new(),
        }
    }
}

use std::marker::PhantomData;

use necsim_core::{
    event::{EventType, PackedEvent},
    reporter::{EventFilter, Reporter},
};

use necsim_impls_no_std::reporter::ReporterContext;

#[allow(clippy::module_name_repetitions)]
pub struct WaterLevelReporter<'e, R: ReporterContext> {
    water_level: f64,
    slow_events: &'e mut Vec<PackedEvent>,
    fast_events: &'e mut Vec<PackedEvent>,
    _marker: PhantomData<R>,
}

impl<'e, R: ReporterContext> EventFilter for WaterLevelReporter<'e, R> {
    const REPORT_DISPERSAL: bool = R::Reporter::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = R::Reporter::REPORT_SPECIATION;
}

impl<'e, R: ReporterContext> Reporter for WaterLevelReporter<'e, R> {
    #[inline]
    fn report_event(&mut self, event: &PackedEvent) {
        if (Self::REPORT_SPECIATION && matches!(event.r#type(), EventType::Speciation))
            || (Self::REPORT_DISPERSAL && matches!(event.r#type(), EventType::Dispersal { .. }))
        {
            if event.time() < self.water_level {
                self.slow_events.push(event.clone())
            } else {
                self.fast_events.push(event.clone())
            }
        }
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // Ignore the progress from the individual simulations
    }
}

impl<'e, R: ReporterContext> WaterLevelReporter<'e, R> {
    pub fn new(
        water_level: f64,
        slow_events: &'e mut Vec<PackedEvent>,
        fast_events: &'e mut Vec<PackedEvent>,
    ) -> Self {
        Self {
            water_level,
            slow_events,
            fast_events,
            _marker: PhantomData::<R>,
        }
    }
}

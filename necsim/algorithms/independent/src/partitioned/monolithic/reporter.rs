use std::marker::PhantomData;

use necsim_core::{event::PackedEvent, impl_report, reporter::Reporter};

use necsim_impls_no_std::reporter::ReporterContext;

#[allow(clippy::module_name_repetitions)]
pub struct WaterLevelReporter<'e, R: ReporterContext> {
    water_level: f64,
    slow_events: &'e mut Vec<PackedEvent>,
    fast_events: &'e mut Vec<PackedEvent>,
    _marker: PhantomData<R>,
}

impl<'e, R: ReporterContext> Reporter for WaterLevelReporter<'e, R> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<
        <<R as ReporterContext>::Reporter as Reporter
    >::ReportSpeciation> {
        event.maybe_use_in(|event| {
            if event.time < self.water_level {
                self.slow_events.push(event.clone().into())
            } else {
                self.fast_events.push(event.clone().into())
            }
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<
            <<R as ReporterContext>::Reporter as Reporter
        >::ReportDispersal> {
        event.maybe_use_in(|event| {
            if event.time < self.water_level {
                self.slow_events.push(event.clone().into())
            } else {
                self.fast_events.push(event.clone().into())
            }
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
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

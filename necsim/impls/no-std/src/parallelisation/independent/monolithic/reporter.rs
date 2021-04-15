use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};

use necsim_core::{event::PackedEvent, impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct WaterLevelReporter<'e, R: Reporter> {
    water_level: f64,
    slow_events: &'e mut Vec<PackedEvent>,
    fast_events: &'e mut Vec<PackedEvent>,
    _marker: PhantomData<R>,
}

impl<'e, R: Reporter> fmt::Debug for WaterLevelReporter<'e, R> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<PackedEvent; {}>", self.0)
            }
        }

        fmt.debug_struct("PartitionReporterProxy")
            .field("water_level", &self.water_level)
            .field("slow_events", &EventBufferLen(self.slow_events.len()))
            .field("fast_events", &EventBufferLen(self.fast_events.len()))
            .finish()
    }
}

impl<'e, R: Reporter> Reporter for WaterLevelReporter<'e, R> {
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<R::ReportSpeciation> {
        event.maybe_use_in(|event| {
            if event.event_time < self.water_level {
                self.slow_events.push(event.clone().into())
            } else {
                self.fast_events.push(event.clone().into())
            }
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<R::ReportDispersal> {
        event.maybe_use_in(|event| {
            if event.event_time < self.water_level {
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

impl<'e, R: Reporter> WaterLevelReporter<'e, R> {
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

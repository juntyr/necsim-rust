use alloc::vec::Vec;
use core::{fmt, marker::PhantomData};
use necsim_core_bond::NonNegativeF64;

use necsim_core::{
    event::{PackedEvent, TypedEvent},
    impl_report,
    reporter::{used::Unused, Reporter},
};

use crate::partitioning::LocalPartition;

use super::WaterLevelReporterProxy;

#[allow(clippy::module_name_repetitions)]
pub struct LiveWaterLevelReporterProxy<'p, R: Reporter, P: LocalPartition<R>> {
    water_level: NonNegativeF64,
    slow_events: Vec<PackedEvent>,
    fast_events: Vec<PackedEvent>,

    local_partition: &'p mut P,
    _marker: PhantomData<R>,
}

impl<'p, R: Reporter, P: LocalPartition<R>> fmt::Debug for LiveWaterLevelReporterProxy<'p, R, P> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct EventBufferLen(usize);

        impl fmt::Debug for EventBufferLen {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                write!(f, "Vec<PackedEvent; {}>", self.0)
            }
        }

        fmt.debug_struct("LiveWaterLevelReporterProxy")
            .field("water_level", &self.water_level)
            .field("slow_events", &EventBufferLen(self.slow_events.len()))
            .field("fast_events", &EventBufferLen(self.fast_events.len()))
            .finish()
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Reporter for LiveWaterLevelReporterProxy<'p, R, P> {
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

#[contract_trait]
impl<'p, R: Reporter, P: LocalPartition<R>> WaterLevelReporterProxy<'p, R, P>
    for LiveWaterLevelReporterProxy<'p, R, P>
{
    fn new(capacity: usize, local_partition: &'p mut P) -> Self {
        info!("Events will be reported using the live water-level algorithm ...");

        Self {
            water_level: NonNegativeF64::zero(),
            slow_events: Vec::with_capacity(capacity),
            fast_events: Vec::with_capacity(capacity),

            local_partition,
            _marker: PhantomData::<R>,
        }
    }

    fn water_level(&self) -> NonNegativeF64 {
        self.water_level
    }

    fn advance_water_level(&mut self, water_level: NonNegativeF64) {
        // Report all events below the water level in sorted order
        self.slow_events.sort();

        for event in self.slow_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(Unused::new(&event));
                },
            }
        }

        // Update the water level
        self.water_level = water_level;

        // Move fast events below the new water level into slow events
        self.slow_events.extend(
            self.fast_events
                .drain_filter(|event| event.event_time < water_level),
        );
    }

    fn local_partition(&mut self) -> &mut P {
        self.local_partition
    }
}

impl<'p, R: Reporter, P: LocalPartition<R>> Drop for LiveWaterLevelReporterProxy<'p, R, P> {
    fn drop(&mut self) {
        // Report all events below the water level in sorted order
        self.slow_events.sort();

        for event in self.slow_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(Unused::new(&event));
                },
            }
        }

        // Report all events above the water level in sorted order
        self.fast_events.sort();

        for event in self.fast_events.drain(..) {
            match event.into() {
                TypedEvent::Speciation(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_speciation(Unused::new(&event));
                },
                TypedEvent::Dispersal(event) => {
                    self.local_partition
                        .get_reporter()
                        .report_dispersal(Unused::new(&event));
                },
            }
        }
    }
}

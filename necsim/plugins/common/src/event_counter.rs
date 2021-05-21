use std::{fmt, fmt::Write};

use necsim_core::{
    event::{DispersalEvent, LineageInteraction, SpeciationEvent},
    impl_finalise, impl_report,
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

#[allow(clippy::module_name_repetitions)]
pub struct EventCounterReporter {
    last_parent_prior_time: Option<NonNegativeF64>,
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    raw_total: usize,
    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
    late_dispersal: usize,
    late_coalescence: usize,
}

impl fmt::Debug for EventCounterReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("EventReporter")
            .field("speciation", &self.speciation)
            .field("out_dispersal", &self.out_dispersal)
            .field("self_dispersal", &self.self_dispersal)
            .field("out_coalescence", &self.out_coalescence)
            .field("self_coalescence", &self.self_coalescence)
            .field("late_coalescence", &self.late_coalescence)
            .finish()
    }
}

impl<'de> serde::Deserialize<'de> for EventCounterReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for EventCounterReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            self.raw_total += 1;

            if Some(event) == self.last_speciation_event.as_ref() {
                if Some(event.prior_time) != self.last_parent_prior_time {
                    self.late_coalescence += 1;
                }
                self.last_parent_prior_time = Some(event.prior_time);

                return;
            }
            self.last_speciation_event = Some(event.clone());
            self.last_parent_prior_time = Some(event.prior_time);

            self.speciation += 1;
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            self.raw_total += 1;

            if Some(event) == self.last_dispersal_event.as_ref() {
                if Some(event.prior_time) != self.last_parent_prior_time {
                    self.late_coalescence += 1;
                }
                self.last_parent_prior_time = Some(event.prior_time);

                return;
            }
            self.last_dispersal_event = Some(event.clone());
            self.last_parent_prior_time = Some(event.prior_time);

            let self_dispersal = event.origin == event.target;
            let coalescence = match event.interaction {
                LineageInteraction::Coalescence(_) => true,
                LineageInteraction::Maybe => {
                    self.late_dispersal += 1;
                    return
                },
                LineageInteraction::None => false,
            };

            match (self_dispersal, coalescence) {
                (true, true) => {
                    self.self_coalescence += 1;
                },
                (true, false) => {
                    self.self_dispersal += 1;
                },
                (false, true) => {
                    self.out_coalescence += 1;
                },
                (false, false) => {
                    self.out_dispersal += 1;
                },
            }
        })
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });

    impl_finalise!((self) {
        if self.last_speciation_event.is_none() && self.last_dispersal_event.is_none() {
            return;
        }

        let mut event_summary = String::new();

        let _ = writeln!(&mut event_summary, "Event Summary:");

        let _ = writeln!(
            &mut event_summary,
            " - Total #individuals:\n   {}",
            self.speciation + self.self_coalescence + self.out_coalescence + self.late_coalescence
        );
        let _ = writeln!(
            &mut event_summary,
            " - Total #events:\
            \n   - raw:\n     {}\
            \n   - deduplicated:\n     {}",
            self.raw_total,
            self.speciation
                + self.self_coalescence
                + self.out_coalescence
                + self.self_dispersal
                + self.out_dispersal
                + self.late_dispersal
                - self.late_coalescence
        );
        let _ = writeln!(
            &mut event_summary,
            " - Speciation:\
            \n    {}",
            self.speciation
        );
        let _ = writeln!(
            &mut event_summary,
            " - Dispersal:\
            \n   - same location, no coalescence:\n     {}\
            \n   - same location, with coalescence:\n     {}\
            \n   - new location, no coalescence:\n     {}\
            \n   - different location, with coalescence:\n     {}\
            \n   - detected late, no coalescence:\n     {}\
            \n   - detected late, with coalescence:\n     {}",
            self.self_dispersal,
            self.self_coalescence,
            self.out_dispersal,
            self.out_coalescence,
            self.late_dispersal - self.late_coalescence,
            self.late_coalescence
        );

        log::info!("{}", event_summary)
    });
}

impl Default for EventCounterReporter {
    fn default() -> Self {
        Self {
            last_parent_prior_time: None,
            last_speciation_event: None,
            last_dispersal_event: None,

            raw_total: 0,
            speciation: 0,
            out_dispersal: 0,
            self_dispersal: 0,
            out_coalescence: 0,
            self_coalescence: 0,
            late_dispersal: 0,
            late_coalescence: 0,
        }
    }
}

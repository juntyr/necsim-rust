use std::{fmt, fmt::Write};

use necsim_core::{
    event::{DispersalEvent, LineageInteraction, SpeciationEvent},
    impl_finalise, impl_report,
    reporter::Reporter,
};

#[allow(clippy::module_name_repetitions)]
pub struct EventCounterReporter {
    last_speciation_event: Option<SpeciationEvent>,
    last_dispersal_event: Option<DispersalEvent>,

    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
}

impl fmt::Debug for EventCounterReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("EventReporter")
            .field("speciation", &self.speciation)
            .field("out_dispersal", &self.out_dispersal)
            .field("self_dispersal", &self.self_dispersal)
            .field("out_coalescence", &self.out_coalescence)
            .field("self_coalescence", &self.self_coalescence)
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
            if Some(event) == self.last_speciation_event.as_ref() {
                return;
            }
            self.last_speciation_event = Some(event.clone());

            self.speciation += 1;
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            if Some(event) == self.last_dispersal_event.as_ref() {
                return;
            }
            self.last_dispersal_event = Some(event.clone());

            let self_dispersal = event.origin == event.target;
            let coalescence = matches!(event.interaction, LineageInteraction::Coalescence(_));

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
            " - Total #individuals:\n     {}",
            self.speciation + self.self_coalescence + self.out_coalescence
        );
        let _ = writeln!(
            &mut event_summary,
            " - Total #events:\n     {}",
            self.speciation
                + self.self_coalescence
                + self.out_coalescence
                + self.self_dispersal
                + self.out_dispersal
        );

        let _ = writeln!(
            &mut event_summary,
            " - Speciation:\n     {}",
            self.speciation
        );
        let _ = writeln!(
            &mut event_summary,
            " - Dispersal outside cell:\n     {}",
            self.out_dispersal
        );
        let _ = writeln!(
            &mut event_summary,
            " - Dispersal inside cell:\n     {}",
            self.self_dispersal
        );
        let _ = writeln!(
            &mut event_summary,
            " - Coalescence outside cell:\n     {}",
            self.out_coalescence
        );
        let _ = write!(
            &mut event_summary,
            " - Coalescence inside cell:\n     {}",
            self.self_coalescence
        );

        log::info!("{}", event_summary)
    });
}

impl Default for EventCounterReporter {
    #[debug_ensures(
        ret.speciation == 0 &&
        ret.out_dispersal == 0 &&
        ret.self_dispersal == 0 &&
        ret.out_coalescence == 0 &&
        ret.self_coalescence == 0,
        "initialises all events to 0"
    )]
    fn default() -> Self {
        Self {
            last_speciation_event: None,
            last_dispersal_event: None,

            speciation: 0,
            out_dispersal: 0,
            self_dispersal: 0,
            out_coalescence: 0,
            self_coalescence: 0,
        }
    }
}

use necsim_core::event_generator::{Event, EventType};
use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct EventReporter {
    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
}

impl Reporter for EventReporter {
    fn report_event(&mut self, event: &Event) {
        match event.r#type() {
            EventType::Speciation => {
                self.speciation += 1;
            }
            EventType::Dispersal {
                origin,
                target,
                coalescence,
                ..
            } => {
                let self_dispersal = origin == target;

                match (self_dispersal, coalescence) {
                    (true, true) => {
                        self.self_coalescence += 1;
                    }
                    (true, false) => {
                        self.self_dispersal += 1;
                    }
                    (false, true) => {
                        self.out_coalescence += 1;
                    }
                    (false, false) => {
                        self.out_dispersal += 1;
                    }
                }
            }
        }
    }
}

impl Default for EventReporter {
    fn default() -> Self {
        Self {
            speciation: 0,
            out_dispersal: 0,
            self_dispersal: 0,
            out_coalescence: 0,
            self_coalescence: 0,
        }
    }
}

impl EventReporter {
    pub fn report(self) {
        println!("{:=^80}", " Event Summary ");

        println!("Speciation:\n\t{}", self.speciation);
        println!(
            "Dispersal outside cell without coalescence:\n\t{}",
            self.out_dispersal
        );
        println!(
            "Dispersal inside cell without coalescence:\n\t{}",
            self.self_dispersal
        );
        println!(
            "Dispersal outside cell with coalescence:\n\t{}",
            self.out_coalescence
        );
        println!(
            "Dispersal inside cell with coalescence:\n\t{}",
            self.self_coalescence
        );

        println!("{:=^80}", " Event Summary ");
    }
}

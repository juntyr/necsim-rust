use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

mod contract;

#[allow(clippy::module_name_repetitions)]
pub struct EventReporter {
    speciation: usize,
    out_dispersal: usize,
    self_dispersal: usize,
    out_coalescence: usize,
    self_coalescence: usize,
}

impl EventFilter for EventReporter {
    const REPORT_DISPERSAL: bool = true;
    const REPORT_SPECIATION: bool = true;
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for EventReporter {
    #[debug_ensures(contract::explicit_event_reporter_report_event_contract(
        event.origin(), event.r#type(), old(self.speciation), old(self.out_dispersal),
        old(self.self_dispersal), old(self.out_coalescence), old(self.self_coalescence),
        self.speciation, self.out_dispersal, self.self_dispersal, self.out_coalescence,
        self.self_coalescence
    ), "counts all distinct event types without changing unaffected counts")]
    fn report_event(&mut self, event: &Event<H, R>) {
        match event.r#type() {
            EventType::Speciation => {
                self.speciation += 1;
            },
            EventType::Dispersal {
                target,
                coalescence,
                ..
            } => {
                let self_dispersal = event.origin() == target;
                let coalescence = coalescence.is_some();

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
            },
        }
    }
}

impl Default for EventReporter {
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

        println!(
            "Total #individuals:\n\t{}",
            self.speciation + self.self_coalescence + self.out_coalescence
        );
        println!(
            "Total #events:\n\t{}",
            self.speciation
                + self.self_coalescence
                + self.out_coalescence
                + self.self_dispersal
                + self.out_dispersal
        );

        println!("Speciation:\n\t{}", self.speciation);
        println!("Dispersal outside cell:\n\t{}", self.out_dispersal);
        println!("Dispersal inside cell:\n\t{}", self.self_dispersal);
        println!("Coalescence outside cell:\n\t{}", self.out_coalescence);
        println!("Coalescence inside cell:\n\t{}", self.self_coalescence);

        println!("{:=^80}", " Event Summary ");
    }
}

use necsim_core::{
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

#[allow(clippy::module_name_repetitions)]
pub struct BiodiversityReporter {
    last_event: Option<Event>,

    biodiversity: usize,
}

impl EventFilter for BiodiversityReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = true;
}

impl Reporter for BiodiversityReporter {
    #[debug_ensures(if old(Some(event) == self.last_event.as_ref()) {
        match event.r#type() {
            EventType::Speciation => self.biodiversity == old(self.biodiversity) + 1,
            _ => self.biodiversity == old(self.biodiversity),
        }
    } else { true }, "EventType::Speciation increments self.biodiversity")]
    fn report_event(&mut self, event: &Event) {
        if Some(event) == self.last_event.as_ref() {
            return;
        }
        self.last_event = Some(event.clone());

        if let EventType::Speciation = event.r#type() {
            self.biodiversity += 1;
        }
    }
}

impl Default for BiodiversityReporter {
    #[debug_ensures(ret.biodiversity == 0, "biodiversity initialised to 0")]
    fn default() -> Self {
        Self {
            last_event: None,
            biodiversity: 0,
        }
    }
}

impl BiodiversityReporter {
    #[must_use]
    #[debug_ensures(ret == self.biodiversity, "returns biodiversity")]
    pub fn biodiversity(self) -> usize {
        self.biodiversity
    }
}

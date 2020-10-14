use necsim_core::event_generator::{Event, EventType};
use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct BiodiversityReporter {
    biodiversity: usize,
}

impl Reporter for BiodiversityReporter {
    fn report_event(&mut self, event: &Event) {
        if let EventType::Speciation = event.r#type() {
            self.biodiversity += 1;
        }
    }
}

impl Default for BiodiversityReporter {
    fn default() -> Self {
        Self { biodiversity: 0 }
    }
}

impl BiodiversityReporter {
    #[must_use]
    pub fn biodiversity(self) -> usize {
        self.biodiversity
    }
}

use necsim_core::event_generator::{Event, EventType};
use necsim_core::lineage::LineageReference;
use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct BiodiversityReporter {
    biodiversity: usize,
}

impl Reporter for BiodiversityReporter {
    #[debug_ensures(match event.r#type() {
        EventType::Speciation => self.biodiversity == old(self.biodiversity) + 1,
        _ => self.biodiversity == old(self.biodiversity),
    }, "EventType::Speciation increments self.biodiversity")]
    fn report_event(&mut self, event: &Event<impl LineageReference>) {
        if let EventType::Speciation = event.r#type() {
            self.biodiversity += 1;
        }
    }
}

impl Default for BiodiversityReporter {
    #[debug_ensures(ret.biodiversity == 0, "biodiversity initialised to 0")]
    fn default() -> Self {
        Self { biodiversity: 0 }
    }
}

impl BiodiversityReporter {
    #[must_use]
    #[debug_ensures(ret == self.biodiversity, "returns biodiversity")]
    pub fn biodiversity(self) -> usize {
        self.biodiversity
    }
}

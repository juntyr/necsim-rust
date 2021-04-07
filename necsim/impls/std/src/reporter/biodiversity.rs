use necsim_core::{event::SpeciationEvent, impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct BiodiversityReporter {
    last_event: Option<SpeciationEvent>,

    biodiversity: usize,
}

impl Reporter for BiodiversityReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            if Some(event) == self.last_event.as_ref() {
                return;
            }

            self.last_event = Some(event.clone());

            self.biodiversity += 1;
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
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

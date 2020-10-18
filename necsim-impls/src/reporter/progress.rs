use necsim_core::event_generator::{Event, EventType};
use necsim_core::lineage::LineageReference;
use necsim_core::reporter::Reporter;

use indicatif::{ProgressBar, ProgressStyle};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    progress: ProgressBar,
}

impl Reporter for ProgressReporter {
    #[debug_ensures(match event.r#type() {
        EventType::Speciation | EventType::Dispersal {
            coalescence: Some(_),
            ..
        } => {
            self.progress.position() == old(self.progress.position()) + 1
        },
        _ => self.progress.position() == old(self.progress.position()),
    }, "only speciation and coalescence increment the progress")]
    fn report_event(&mut self, event: &Event<impl LineageReference>) {
        match event.r#type() {
            EventType::Speciation
            | EventType::Dispersal {
                coalescence: Some(_),
                ..
            } => self.progress.inc(1),
            _ => (),
        }
    }
}

impl ProgressReporter {
    #[must_use]
    pub fn new(total: u64) -> Self {
        let progress = ProgressBar::new(total).with_style(
            ProgressStyle::default_bar()
                .template("    [{elapsed_precise}] {bar:50.cyan/blue} [{eta_precise}]    "),
        );

        progress.set_draw_delta(total / 200);

        Self { progress }
    }

    pub fn finish(self) {
        self.progress.finish()
    }
}

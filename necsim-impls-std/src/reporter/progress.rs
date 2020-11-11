use necsim_core::cogs::{Habitat, LineageReference};
use necsim_core::event::{Event, EventType};
use necsim_core::reporter::Reporter;

use indicatif::{ProgressBar, ProgressStyle};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    progress: ProgressBar,
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for ProgressReporter {
    const REPORT_SPECIATION: bool = true;
    const REPORT_DISPERSAL: bool = false;

    #[debug_ensures(match event.r#type() {
        EventType::Speciation | EventType::Dispersal {
            coalescence: Some(_),
            ..
        } => {
            self.progress.position() == old(self.progress.position()) + 1
        },
        _ => self.progress.position() == old(self.progress.position()),
    }, "only speciation and coalescence increment the progress")]
    fn report_event(&mut self, event: &Event<H, R>) {
        if self.progress.position() == 0 {
            self.progress.reset();
        }

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

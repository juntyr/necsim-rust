use necsim_core::{
    cogs::{Habitat, LineageReference},
    event::{Event, EventType},
    reporter::{EventFilter, Reporter},
};

use indicatif::{ProgressBar, ProgressStyle};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    progress: ProgressBar,
}

impl EventFilter for ProgressReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = true;
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for ProgressReporter {
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

        progress.enable_steady_tick(100);

        Self { progress }
    }

    pub fn finish(self) {
        self.progress.finish()
    }
}

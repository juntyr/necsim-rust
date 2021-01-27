use std::cmp::Ordering;

use indicatif::{ProgressBar, ProgressStyle};

use necsim_core::reporter::{EventFilter, Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    progress: ProgressBar,
    last_remaining: u64,
    total: u64,
}

impl EventFilter for ProgressReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = false;
}

impl Reporter for ProgressReporter {
    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        match remaining.cmp(&self.last_remaining) {
            Ordering::Greater => {
                self.total += remaining - self.last_remaining;

                self.progress.set_length(self.total);
                self.progress.set_position(self.total - remaining);
            },
            Ordering::Less => {
                self.progress.inc(self.last_remaining - remaining);
            },
            Ordering::Equal => (),
        }

        self.last_remaining = remaining;
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

        Self {
            progress,

            last_remaining: 0_u64,
            total: 0_u64,
        }
    }

    pub fn finish(self) {
        self.progress.finish()
    }
}

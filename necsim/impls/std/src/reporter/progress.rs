use std::cmp::Ordering;

use indicatif::{ProgressBar, ProgressStyle};

use necsim_core::reporter::{EventFilter, Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    progress: Option<ProgressBar>,
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
        // TODO: Show the progress bar in MPI mode as well
        let progress = self.progress.get_or_insert_with(|| {
            let progress =
                ProgressBar::new(1)
                    .with_style(ProgressStyle::default_bar().template(
                        "    [{elapsed_precise}] {bar:50.cyan/blue} [{eta_precise}]    ",
                    ));

            progress.enable_steady_tick(100);

            progress
        });

        match remaining.cmp(&self.last_remaining) {
            Ordering::Greater => {
                self.total += remaining - self.last_remaining;

                progress.set_length(self.total);
                progress.set_position(self.total - remaining);
            },
            Ordering::Less => {
                progress.inc(self.last_remaining - remaining);
            },
            Ordering::Equal => (),
        }

        self.last_remaining = remaining;
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self {
            progress: None,
            last_remaining: 0_u64,
            total: 0_u64,
        }
    }
}

impl ProgressReporter {
    pub fn finish(self) {
        if let Some(progress) = self.progress {
            progress.finish()
        }
    }
}

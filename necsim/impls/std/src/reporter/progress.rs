use std::io::{self, Write};

use necsim_core::reporter::{EventFilter, Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    last_filter: u64,
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
        // Update the progress total in case of regression
        if self.last_remaining < remaining {
            self.total += remaining - self.last_remaining;
        }

        let filter = remaining * Self::FILTER_PRECISION / self.last_filter.max(1_u64);

        // Filter out updates which do not change the progress significantly
        // The filter gets more detailed towards the end of the task
        if (filter + 1) < Self::FILTER_PRECISION || filter > Self::FILTER_PRECISION {
            self.last_filter = remaining;

            #[allow(clippy::cast_possible_truncation)]
            let display_progress =
                ((self.total - remaining) * (Self::UPDATE_PRECISION as u64) / self.total) as usize;

            // Display a simple progress bar to stderr

            eprint!("\r{:>13} [", self.total - remaining);
            if display_progress == 0 {
                eprint!("{:>rest$}", "", rest = (Self::UPDATE_PRECISION));
            } else if remaining > 0 {
                eprint!(
                    "{:=<progress$}>{:>rest$}",
                    "",
                    "",
                    progress = (display_progress - 1),
                    rest = (Self::UPDATE_PRECISION - display_progress)
                );
            } else {
                eprint!("{:=<progress$}", "", progress = (Self::UPDATE_PRECISION));
            }
            eprint!("] {:<13}", self.total);

            if remaining == 0 {
                eprint!("\r\n");
            }

            // Flush stderr to update the progress bar
            let _ = io::stderr().flush();
        }

        self.last_remaining = remaining;
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        Self {
            last_filter: 0_u64,
            last_remaining: 0_u64,
            total: 0_u64,
        }
    }
}

impl ProgressReporter {
    const FILTER_PRECISION: u64 = 100;
    const UPDATE_PRECISION: usize = 50;

    pub fn finish(self) {
        std::mem::drop(self)
    }
}

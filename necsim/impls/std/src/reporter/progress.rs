use std::{
    io::{self, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Sender, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use necsim_core::reporter::{EventFilter, Reporter};

struct ProgressUpdater {
    thread: JoinHandle<()>,
    sender: Sender<()>,
}

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    updater: Option<ProgressUpdater>,
    last_remaining: Arc<AtomicU64>,
    total: Arc<AtomicU64>,
}

impl EventFilter for ProgressReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = false;
}

impl Reporter for ProgressReporter {
    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        let last_remaining = self.last_remaining.swap(remaining, Ordering::AcqRel);

        // Update the progress total in case of regression
        if last_remaining < remaining {
            self.total
                .fetch_add(remaining - last_remaining, Ordering::AcqRel);
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        let last_remaining = Arc::new(AtomicU64::new(0_u64));
        let last_total = Arc::new(AtomicU64::new(0_u64));

        let remaining = Arc::clone(&last_remaining);
        let total = Arc::clone(&last_total);

        let (sender, receiver) = mpsc::channel();

        let thread = thread::spawn(move || {
            loop {
                thread::sleep(Duration::from_millis(100));

                match receiver.try_recv() {
                    Ok(_) | Err(TryRecvError::Disconnected) => break,
                    Err(TryRecvError::Empty) => {},
                }

                let total = total.load(Ordering::Acquire);

                if total > 0 {
                    display_progress(total, remaining.load(Ordering::Acquire).min(total));

                    // Flush stderr to update the progress bar
                    let _ = io::stderr().flush();
                }
            }
        });

        Self {
            updater: Some(ProgressUpdater { thread, sender }),
            last_remaining,
            total: last_total,
        }
    }
}

impl ProgressReporter {
    pub fn finish(mut self) {
        if let Some(updater) = self.updater.take() {
            if updater.sender.send(()).is_ok() {
                let _ = updater.thread.join();
            }
        }

        let total = self.total.load(Ordering::Acquire);

        if total > 0 {
            display_progress(
                total,
                self.last_remaining.load(Ordering::Acquire).min(total),
            );

            eprint!("\r\n");

            // Flush stderr to update the progress bar
            let _ = io::stderr().flush();
        }

        std::mem::drop(self)
    }
}

fn display_progress(total: u64, remaining: u64) {
    const UPDATE_PRECISION: usize = 50;

    #[allow(clippy::cast_possible_truncation)]
    let display_progress =
        ((total - remaining) * (UPDATE_PRECISION as u64) / total.max(1)) as usize;

    // Display a simple progress bar to stderr
    eprint!("\r{:>13} [", total - remaining);
    if display_progress == 0 {
        eprint!("{:>rest$}", "", rest = (UPDATE_PRECISION));
    } else if remaining > 0 {
        eprint!(
            "{:=<progress$}>{:>rest$}",
            "",
            "",
            progress = (display_progress - 1),
            rest = (UPDATE_PRECISION - display_progress)
        );
    } else {
        eprint!("{:=<progress$}", "", progress = (UPDATE_PRECISION));
    }
    eprint!("] {:<13}", total);
}

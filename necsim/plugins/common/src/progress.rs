use std::{
    fmt,
    io::{self, Write},
    sync::{
        atomic::{AtomicU64, Ordering},
        mpsc::{self, Sender, TryRecvError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::Duration,
};

use necsim_core::{impl_finalise, impl_report, reporter::Reporter};

struct ProgressUpdater {
    thread: JoinHandle<()>,
    sender: Sender<()>,
}

#[allow(clippy::module_name_repetitions)]
pub struct ProgressReporter {
    updater: Option<ProgressUpdater>,
    last_remaining: Arc<AtomicU64>,
    last_total: Arc<AtomicU64>,
}

impl fmt::Debug for ProgressReporter {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ProgressReporter")
            .field("last_remaining", &self.last_remaining)
            .field("last_total", &self.last_total)
            .finish()
    }
}

impl<'de> serde::Deserialize<'de> for ProgressReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for ProgressReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Used {
        remaining.use_in(|remaining| {
            let last_remaining = self.last_remaining.swap(*remaining, Ordering::AcqRel);

            // Update the progress total in case of regression
            if last_remaining < *remaining {
                self.last_total
                    .fetch_add(remaining - last_remaining, Ordering::AcqRel);
            }

            if last_remaining > 0 && *remaining == 0 {
                let total = self.last_total.load(Ordering::Acquire);

                display_progress(total, self.last_remaining.load(Ordering::Acquire).min(total));

                // Flush stderr to update the progress bar
                std::mem::drop(io::stderr().flush());
            }
        })
    });

    impl_finalise!((self) {
        std::mem::drop(self)
    });

    fn initialise(&mut self) -> Result<(), String> {
        if self.updater.is_some() {
            return Ok(());
        }

        let remaining = Arc::clone(&self.last_remaining);
        let total = Arc::clone(&self.last_total);

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
                    std::mem::drop(io::stderr().flush());
                }
            }
        });

        self.updater = Some(ProgressUpdater { thread, sender });

        Ok(())
    }
}

impl Drop for ProgressReporter {
    fn drop(&mut self) {
        if let Some(updater) = self.updater.take() {
            if updater.sender.send(()).is_ok() {
                std::mem::drop(updater.thread.join());
            }
        }
    }
}

impl Default for ProgressReporter {
    fn default() -> Self {
        let last_remaining = Arc::new(AtomicU64::new(0_u64));
        let last_total = Arc::new(AtomicU64::new(0_u64));

        Self {
            updater: None,
            last_remaining,
            last_total,
        }
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

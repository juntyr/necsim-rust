use std::time::{Duration, Instant};

use necsim_core::{impl_report, reporter::Reporter};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ExecutionTimeReporter {
    init_time: Instant,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
}

impl<'de> serde::Deserialize<'de> for ExecutionTimeReporter {
    fn deserialize<D: serde::Deserializer<'de>>(_deserializer: D) -> Result<Self, D::Error> {
        Ok(Self::default())
    }
}

impl Reporter for ExecutionTimeReporter {
    impl_report!(speciation(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(
        #[debug_ensures(self.start_time.is_some(), "start_time is set after first call")]
        progress(&mut self, remaining: Unused) -> Used {
            remaining.use_in(|remaining| {
                if self.start_time.is_none() {
                    self.start_time = Some(Instant::now());
                }

                if *remaining == 0 {
                    self.end_time = Some(Instant::now());
                }
            })
        }
    );
}

impl Default for ExecutionTimeReporter {
    #[debug_ensures(ret.start_time.is_none(), "initialises start_time to None")]
    fn default() -> Self {
        Self {
            init_time: Instant::now(),
            start_time: None,
            end_time: None,
        }
    }
}

pub struct ExecutionTime {
    pub initialisation: Duration,
    pub execution: Duration,
    pub cleanup: Duration,
}

impl ExecutionTimeReporter {
    #[must_use]
    pub fn execution_time(self) -> Option<ExecutionTime> {
        if let (Some(start_time), Some(end_time)) = (self.start_time, self.end_time) {
            Some(ExecutionTime {
                initialisation: start_time - self.init_time,
                execution: end_time - start_time,
                cleanup: end_time.elapsed(),
            })
        } else {
            None
        }
    }
}

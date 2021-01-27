use std::time::{Duration, Instant};

use necsim_core::reporter::{EventFilter, Reporter};

#[allow(clippy::module_name_repetitions)]
pub struct ExecutionTimeReporter {
    init_time: Instant,
    start_time: Option<Instant>,
    end_time: Option<Instant>,
}

impl EventFilter for ExecutionTimeReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = false;
}

impl Reporter for ExecutionTimeReporter {
    #[debug_ensures(self.start_time.is_some(), "start_time is set after first call")]
    #[debug_ensures(self.end_time.is_some(), "end_time is set")]
    #[debug_ensures(
        old(self.start_time).is_some() -> old(self.start_time) == self.start_time,
        "only updates start_time on first call"
    )]
    fn report_progress(&mut self, _remaining: u64) {
        let now = Instant::now();

        self.start_time.get_or_insert(now);
        self.end_time = Some(now);
    }
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

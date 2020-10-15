use std::time::{Duration, Instant};

use necsim_core::event_generator::Event;
use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct ExecutionTimeReporter {
    start_time: Option<Instant>,
}

impl Reporter for ExecutionTimeReporter {
    fn report_event(&mut self, _event: &Event) {
        self.start_time.get_or_insert_with(Instant::now);
    }
}

impl Default for ExecutionTimeReporter {
    fn default() -> Self {
        Self { start_time: None }
    }
}

impl ExecutionTimeReporter {
    #[must_use]
    pub fn execution_time(self) -> Duration {
        self.start_time
            .map_or_else(Default::default, |i| i.elapsed())
    }
}

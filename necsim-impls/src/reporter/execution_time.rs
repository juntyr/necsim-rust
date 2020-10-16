use std::time::{Duration, Instant};

use necsim_core::event_generator::Event;
use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct ExecutionTimeReporter {
    start_time: Option<Instant>,
}

impl Reporter for ExecutionTimeReporter {
    #[debug_ensures(match old(self.start_time) {
        None => self.start_time.is_some(),
        Some(t) => {
            self.start_time.is_some() &&
            t == self.start_time.unwrap()
        },
    })]
    fn report_event(&mut self, _event: &Event) {
        self.start_time.get_or_insert_with(Instant::now);
    }
}

impl Default for ExecutionTimeReporter {
    #[debug_ensures(ret.start_time.is_none())]
    fn default() -> Self {
        Self { start_time: None }
    }
}

impl ExecutionTimeReporter {
    #[must_use]
    #[debug_ensures(match self.start_time {
        None => ret == Duration::default(),
        Some(_) => ret > Duration::default(),
    })]
    pub fn execution_time(self) -> Duration {
        self.start_time
            .map_or_else(Duration::default, |i| i.elapsed())
    }
}

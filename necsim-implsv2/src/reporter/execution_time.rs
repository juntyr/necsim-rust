use std::time::{Duration, Instant};

use necsim_corev2::cogs::{Habitat, LineageReference};
use necsim_corev2::event::Event;
use necsim_corev2::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub struct ExecutionTimeReporter {
    start_time: Option<Instant>,
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for ExecutionTimeReporter {
    #[debug_ensures(self.start_time.is_some(), "start_time is set after first call")]
    #[debug_ensures(
        old(self.start_time).is_some() -> old(self.start_time) == self.start_time,
        "only updates start_time on first call"
    )]
    fn report_event(&mut self, _event: &Event<H, R>) {
        self.start_time.get_or_insert_with(Instant::now);
    }
}

impl Default for ExecutionTimeReporter {
    #[debug_ensures(ret.start_time.is_none(), "initialises start_time to None")]
    fn default() -> Self {
        Self { start_time: None }
    }
}

impl ExecutionTimeReporter {
    #[must_use]
    #[debug_ensures(match self.start_time {
        None => ret == Duration::default(),
        Some(_) => ret > Duration::default(),
    }, "execution_time is zero if no execution, otherwise greater zero")]
    pub fn execution_time(self) -> Duration {
        self.start_time
            .map_or_else(Duration::default, |i| i.elapsed())
    }
}

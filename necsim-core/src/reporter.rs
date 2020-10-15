use crate::event_generator::Event;

pub trait Reporter {
    fn report_event(&mut self, event: &Event);
}

#[allow(clippy::module_name_repetitions)]
pub struct NullReporter;

impl Reporter for NullReporter {
    fn report_event(&mut self, _event: &Event) {
        // no-op
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ReporterCombinator<'r, L: Reporter, R: Reporter> {
    first: &'r mut L,
    second: R,
}

impl<'r, L: Reporter, R: Reporter> Reporter for ReporterCombinator<'r, L, R> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        self.first.report_event(event);
        self.second.report_event(event);
    }
}

impl<'r, L: Reporter, R: Reporter> ReporterCombinator<'r, L, R> {
    #[must_use]
    pub fn new(first: &'r mut L, second: R) -> Self {
        Self { first, second }
    }
}

#[macro_export]
macro_rules! ReporterGroup {
    () => {
        necsim_core::reporter::NullReporter
    };
    ($first_reporter:ident $(,$reporter_tail:ident)*) => {
        {
            necsim_core::reporter::ReporterCombinator::new(
                &mut $first_reporter,
                ReporterGroup![$($reporter_tail),*]
            )
        }
    }
}

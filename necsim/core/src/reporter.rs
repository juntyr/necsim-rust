use crate::event::Event;

pub trait EventFilter {
    const REPORT_SPECIATION: bool;
    const REPORT_DISPERSAL: bool;
}

pub trait Reporter: EventFilter {
    #[inline]
    fn report_event(&mut self, _event: &Event) {
        // no-op
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // no-op
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct NullReporter;

impl EventFilter for NullReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = false;
}

impl Reporter for NullReporter {
    #[inline]
    fn report_event(&mut self, _event: &Event) {
        // no-op
    }

    #[inline]
    fn report_progress(&mut self, _remaining: u64) {
        // no-op
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ReporterCombinator<F: Reporter, T: Reporter> {
    front: F,
    tail: T, // R = ReporterCombinator<...>
}

impl<F: Reporter, T: Reporter> EventFilter for ReporterCombinator<F, T> {
    const REPORT_DISPERSAL: bool = F::REPORT_DISPERSAL || T::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = F::REPORT_SPECIATION || T::REPORT_SPECIATION;
}

impl<F: Reporter, T: Reporter> Reporter for ReporterCombinator<F, T> {
    #[inline]
    fn report_event(&mut self, event: &Event) {
        self.front.report_event(event);
        self.tail.report_event(event);
    }

    #[inline]
    fn report_progress(&mut self, remaining: u64) {
        self.front.report_progress(remaining);
        self.tail.report_progress(remaining);
    }
}

impl<F: Reporter, T: Reporter> ReporterCombinator<F, T> {
    #[must_use]
    /// # Safety
    /// This constructor should not be used directly to combinate reporters.
    /// Use the `ReporterGroup![...]` macro instead.
    pub unsafe fn new(front: F, tail: T) -> Self {
        Self { front, tail }
    }

    #[must_use]
    /// # Safety
    /// This destructor should not be used directly to decompose reporters.
    /// Use the `ReporterUnGroup!{reporter => [...]}` macro instead.
    pub unsafe fn wen(self) -> (F, T) {
        (self.front, self.tail)
    }
}

#[macro_export]
macro_rules! ReporterGroup {
    () => {
        necsim_core::reporter::NullReporter
    };
    ($first_reporter:ident $(,$reporter_tail:ident)*) => {
        {
            unsafe { necsim_core::reporter::ReporterCombinator::new(
                $first_reporter,
                $crate::ReporterGroup![$($reporter_tail),*],
            ) }
        }
    }
}

#[macro_export]
macro_rules! ReporterUnGroup {
    ($reporter:expr => []) => {};
    ($reporter:expr => [$first_reporter:ident $(,$reporter_tail:ident)*]) => {
        {
            let (reporter_front, reporter_tail) = unsafe {
                $reporter.wen()
            };

            $first_reporter = reporter_front;

            $crate::ReporterUnGroup!{reporter_tail => [$($reporter_tail),*]}
        }
    }
}

#[macro_export]
macro_rules! ReporterGroupType {
    () => {
        necsim_core::reporter::NullReporter
    };
    ($first_reporter:ty $(,$reporter_tail:ty)*) => {
        necsim_core::reporter::ReporterCombinator<
            $first_reporter,
            $crate::ReporterGroupType![$($reporter_tail),*],
        >
    }
}

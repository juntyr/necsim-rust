use core::marker::PhantomData;

use crate::{
    cogs::{Habitat, LineageReference},
    event::Event,
};

pub trait EventFilter {
    const REPORT_SPECIATION: bool;
    const REPORT_DISPERSAL: bool;
}

pub trait Reporter<H: Habitat, R: LineageReference<H>>: EventFilter {
    fn report_event(&mut self, event: &Event<H, R>);
}

#[allow(clippy::module_name_repetitions)]
pub struct NullReporter;

impl EventFilter for NullReporter {
    const REPORT_DISPERSAL: bool = false;
    const REPORT_SPECIATION: bool = false;
}

impl<H: Habitat, R: LineageReference<H>> Reporter<H, R> for NullReporter {
    fn report_event(&mut self, _event: &Event<H, R>) {
        // no-op
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct ReporterCombinator<
    H: Habitat,
    R: LineageReference<H>,
    F: Reporter<H, R>,
    T: Reporter<H, R>,
> {
    front: F,
    tail: T, // R = ReporterCombinator<...>
    _marker: PhantomData<(H, R)>,
}

impl<H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>> EventFilter
    for ReporterCombinator<H, R, F, T>
{
    const REPORT_DISPERSAL: bool = F::REPORT_DISPERSAL || T::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = F::REPORT_SPECIATION || T::REPORT_SPECIATION;
}

impl<H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>> Reporter<H, R>
    for ReporterCombinator<H, R, F, T>
{
    #[inline]
    fn report_event(&mut self, event: &Event<H, R>) {
        self.front.report_event(event);
        self.tail.report_event(event);
    }
}

impl<H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>>
    ReporterCombinator<H, R, F, T>
{
    #[must_use]
    /// # Safety
    /// This constructor should not be used directly to combinate reporters.
    /// Use the `ReporterGroup![...]` macro instead.
    pub unsafe fn new(front: F, tail: T) -> Self {
        Self {
            front,
            tail,
            _marker: PhantomData::<(H, R)>,
        }
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
    (<$Habitat:ty, $LineageReference:ty>[]) => {
        necsim_core::reporter::NullReporter
    };
    (<$Habitat:ty, $LineageReference:ty>[$first_reporter:ty $(,$reporter_tail:ty)*]) => {
        necsim_core::reporter::ReporterCombinator<
            $Habitat,
            $LineageReference,
            $first_reporter,
            $crate::ReporterGroupType!{<$Habitat, $LineageReference>[$($reporter_tail),*]},
        >
    }
}

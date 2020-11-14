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
    'r,
    H: Habitat,
    R: LineageReference<H>,
    F: Reporter<H, R>,
    T: Reporter<H, R>,
> {
    front: &'r mut F,
    tail: T, // R = ReporterCombinator<...>
    _marker: PhantomData<(H, R)>,
}

impl<'r, H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>> EventFilter
    for ReporterCombinator<'r, H, R, F, T>
{
    const REPORT_DISPERSAL: bool = F::REPORT_DISPERSAL || T::REPORT_DISPERSAL;
    const REPORT_SPECIATION: bool = F::REPORT_SPECIATION || T::REPORT_SPECIATION;
}

impl<'r, H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>> Reporter<H, R>
    for ReporterCombinator<'r, H, R, F, T>
{
    #[inline]
    fn report_event(&mut self, event: &Event<H, R>) {
        self.front.report_event(event);
        self.tail.report_event(event);
    }
}

impl<'r, H: Habitat, R: LineageReference<H>, F: Reporter<H, R>, T: Reporter<H, R>>
    ReporterCombinator<'r, H, R, F, T>
{
    #[must_use]
    /// # Safety
    /// This constructor should not be used directly to combinate reporters.
    /// Use the `ReporterGroup![]` macro instead.
    pub unsafe fn new(front: &'r mut F, tail: T) -> Self {
        Self {
            front,
            tail,
            _marker: PhantomData::<(H, R)>,
        }
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
                &mut $first_reporter,
                ReporterGroup![$($reporter_tail),*]
            ) }
        }
    }
}

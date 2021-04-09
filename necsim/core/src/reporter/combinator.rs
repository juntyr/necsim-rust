use crate::{
    impl_report,
    reporter::{boolean::Or, Reporter},
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ReporterCombinator<F: Reporter, T: Reporter> {
    front: F,
    tail: T, // R = ReporterCombinator<...>
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

impl<F: Reporter, T: Reporter> Reporter for ReporterCombinator<F, T>
where
    F::ReportSpeciation: Or<T::ReportSpeciation>,
    F::ReportDispersal: Or<T::ReportDispersal>,
    F::ReportProgress: Or<T::ReportProgress>,
{
    impl_report!(speciation(&mut self, event: Unused)
        -> MaybeUsed< <F::ReportSpeciation as Or<T::ReportSpeciation> >::RESULT>
    {
        self.front.report_speciation(event.unused());
        self.tail.report_speciation(event.unused());

        event.into()
    });

    impl_report!(dispersal(&mut self, event: Unused)
        -> MaybeUsed< <F::ReportDispersal as Or<T::ReportDispersal> >::RESULT>
    {
        self.front.report_dispersal(event.unused());
        self.tail.report_dispersal(event.unused());

        event.into()
    });

    impl_report!(progress(&mut self, remaining: Unused)
        -> MaybeUsed< <F::ReportProgress as Or<T::ReportProgress> >::RESULT>
    {
        self.front.report_progress(remaining.unused());
        self.tail.report_progress(remaining.unused());

        remaining.into()
    });

    fn finalise_impl(&mut self) {
        self.front.finalise_impl();
        self.tail.finalise_impl()
    }
}

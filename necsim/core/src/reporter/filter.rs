use core::marker::PhantomData;

use alloc::fmt;

use crate::{
    impl_finalise, impl_report,
    reporter::{
        boolean::{And, Boolean},
        used::Unused,
        Reporter,
    },
};

#[allow(clippy::module_name_repetitions)]
pub struct FilteredReporter<
    R: Reporter,
    KeepSpeciation: Boolean,
    KeepDispersal: Boolean,
    KeepProgress: Boolean,
> {
    reporter: R,
    marker: PhantomData<(KeepSpeciation, KeepDispersal, KeepProgress)>,
}

impl<R: Reporter, KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean> fmt::Debug
    for FilteredReporter<R, KeepSpeciation, KeepDispersal, KeepProgress>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("FilteredReporter")
            .field("reporter", &self.reporter)
            .finish()
    }
}

impl<R: Reporter, KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean> From<R>
    for FilteredReporter<R, KeepSpeciation, KeepDispersal, KeepProgress>
{
    fn from(reporter: R) -> Self {
        Self {
            reporter,
            marker: PhantomData,
        }
    }
}

impl<R: Reporter, KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean> Reporter
    for FilteredReporter<R, KeepSpeciation, KeepDispersal, KeepProgress>
where
    KeepSpeciation: And<R::ReportSpeciation>,
    KeepDispersal: And<R::ReportDispersal>,
    KeepProgress: And<R::ReportProgress>,
{
    impl_report!(speciation(&mut self, event: Unused)
        -> MaybeUsed< <KeepSpeciation as And<R::ReportSpeciation> >::RESULT>
    {
        event.maybe_use_in(|event| {
            self.reporter.report_speciation(Unused::new(event));
        })
    });

    impl_report!(dispersal(&mut self, event: Unused)
        -> MaybeUsed< <KeepDispersal as And<R::ReportDispersal> >::RESULT>
    {
        event.maybe_use_in(|event| {
            self.reporter.report_dispersal(Unused::new(event));
        })
    });

    impl_report!(progress(&mut self, event: Unused)
        -> MaybeUsed< <KeepProgress as And<R::ReportProgress> >::RESULT>
    {
        event.maybe_use_in(|event| {
            self.reporter.report_progress(Unused::new(event));
        })
    });

    impl_finalise!((self) {
        if Self::ReportSpeciation::VALUE || Self::ReportDispersal::VALUE || Self::ReportProgress::VALUE {
            self.reporter.finalise()
        }
    });

    fn initialise(&mut self) -> Result<(), alloc::string::String> {
        if Self::ReportSpeciation::VALUE
            || Self::ReportDispersal::VALUE
            || Self::ReportProgress::VALUE
        {
            self.reporter.initialise()
        } else {
            Ok(())
        }
    }
}

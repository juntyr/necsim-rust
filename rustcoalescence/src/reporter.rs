use std::fmt;

use necsim_core::reporter::{boolean::Boolean, FilteredReporter, Reporter};

use necsim_partitioning_core::context::ReporterContext;

use necsim_plugins_core::import::ReporterPluginVec;

pub struct DynamicReporterContext<
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
    ReportProgress: Boolean,
> {
    reporter: ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean> fmt::Debug
    for DynamicReporterContext<ReportSpeciation, ReportDispersal, ReportProgress>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("DynamicReporterContext")
            .field("reporter", &self.reporter)
            .finish()
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean>
    DynamicReporterContext<ReportSpeciation, ReportDispersal, ReportProgress>
{
    pub fn new(
        reporter: ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>,
    ) -> Self {
        Self { reporter }
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean> ReporterContext
    for DynamicReporterContext<ReportSpeciation, ReportDispersal, ReportProgress>
{
    type Reporter = ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>;

    fn try_build<KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean>(
        self,
    ) -> anyhow::Result<FilteredReporter<Self::Reporter, KeepSpeciation, KeepDispersal, KeepProgress>>
    {
        let mut filtered_reporter = self
            .reporter
            .internal_filter::<KeepSpeciation, KeepDispersal, KeepProgress>();

        match filtered_reporter.initialise() {
            Ok(()) => Ok(FilteredReporter::from(filtered_reporter)),
            Err(err) => Err(anyhow::Error::msg(err)),
        }
    }
}

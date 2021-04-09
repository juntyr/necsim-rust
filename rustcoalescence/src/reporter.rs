use std::fmt;

use necsim_core::reporter::boolean::Boolean;

use necsim_impls_no_std::reporter::ReporterContext;

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

    fn build(self) -> Self::Reporter {
        self.reporter
    }
}

use std::{
    fmt,
    iter::{FromIterator, IntoIterator},
    marker::PhantomData,
};

use necsim_core::{
    impl_report,
    reporter::{
        boolean::{Boolean, False, True},
        used::Unused,
        Reporter,
    },
};

use crate::common::ReporterPlugin;

pub struct ReporterPluginVec<
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
    ReportProgress: Boolean,
> {
    plugins: Box<[ReporterPlugin]>,
    marker: PhantomData<(ReportSpeciation, ReportDispersal, ReportProgress)>,
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean> fmt::Debug
    for ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>
{
    fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt.debug_struct("ReporterPluginVec")
            .field("plugins", &self.plugins)
            .finish()
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean> Reporter
    for ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>
{
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<ReportSpeciation> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.report_speciation {
                    plugin.reporter.report_speciation(Unused::new(event));
                }
            }
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<ReportDispersal> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.report_dispersal {
                    plugin.reporter.report_dispersal(Unused::new(event));
                }
            }
        })
    });

    impl_report!(progress(&mut self, event: Unused) -> MaybeUsed<ReportProgress> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.report_progress {
                    plugin.reporter.report_progress(Unused::new(event));
                }
            }
        })
    });
}

#[allow(clippy::pub_enum_variant_names)]
pub enum AnyReporterPluginVec {
    IgnoreSpeciationIgnoreDispersalIgnoreProgress(ReporterPluginVec<False, False, False>),
    IgnoreSpeciationIgnoreDispersalReportProgress(ReporterPluginVec<False, False, True>),
    IgnoreSpeciationReportDispersalIgnoreProgress(ReporterPluginVec<False, True, False>),
    IgnoreSpeciationReportDispersalReportProgress(ReporterPluginVec<False, True, True>),
    ReportSpeciationIgnoreDispersalIgnoreProgress(ReporterPluginVec<True, False, False>),
    ReportSpeciationIgnoreDispersalReportProgress(ReporterPluginVec<True, False, True>),
    ReportSpeciationReportDispersalIgnoreProgress(ReporterPluginVec<True, True, False>),
    ReportSpeciationReportDispersalReportProgress(ReporterPluginVec<True, True, True>),
}

impl FromIterator<ReporterPlugin> for AnyReporterPluginVec {
    fn from_iter<I: IntoIterator<Item = ReporterPlugin>>(iter: I) -> Self {
        let plugins = iter
            .into_iter()
            .collect::<Vec<ReporterPlugin>>()
            .into_boxed_slice();

        let report_speciation = plugins.iter().any(|reporter| reporter.report_speciation);
        let report_dispersal = plugins.iter().any(|reporter| reporter.report_dispersal);
        let report_progress = plugins.iter().any(|reporter| reporter.report_progress);

        match (report_speciation, report_dispersal, report_progress) {
            (false, false, false) => {
                Self::IgnoreSpeciationIgnoreDispersalIgnoreProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (false, false, true) => {
                Self::IgnoreSpeciationIgnoreDispersalReportProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (false, true, false) => {
                Self::IgnoreSpeciationReportDispersalIgnoreProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (false, true, true) => {
                Self::IgnoreSpeciationReportDispersalReportProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (true, false, false) => {
                Self::ReportSpeciationIgnoreDispersalIgnoreProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (true, false, true) => {
                Self::ReportSpeciationIgnoreDispersalReportProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (true, true, false) => {
                Self::ReportSpeciationReportDispersalIgnoreProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
            (true, true, true) => {
                Self::ReportSpeciationReportDispersalReportProgress(ReporterPluginVec {
                    plugins,
                    marker: PhantomData,
                })
            },
        }
    }
}

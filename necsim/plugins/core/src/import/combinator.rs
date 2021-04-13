use std::{
    fmt,
    iter::{FromIterator, IntoIterator},
    marker::PhantomData,
};

use necsim_core::{
    impl_finalise, impl_report,
    reporter::{
        boolean::{Boolean, False, True},
        used::Unused,
        Reporter,
    },
};

use crate::import::ReporterPlugin;

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
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct("ReporterPluginVec")
            .field("plugins", &self.plugins)
            .finish()
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean>
    ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>
{
    #[must_use]
    pub fn internal_filter<
        KeepSpeciation: Boolean,
        KeepDispersal: Boolean,
        KeepProgress: Boolean,
    >(
        self,
    ) -> Self {
        let mut plugins: Vec<ReporterPlugin> = self.plugins.into_vec();

        plugins.retain(|plugin| {
            (plugin.filter.report_speciation && KeepSpeciation::VALUE)
                || (plugin.filter.report_dispersal && KeepDispersal::VALUE)
                || (plugin.filter.report_progress && KeepProgress::VALUE)
        });

        Self {
            plugins: plugins.into_boxed_slice(),
            marker: self.marker,
        }
    }
}

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean> Reporter
    for ReporterPluginVec<ReportSpeciation, ReportDispersal, ReportProgress>
{
    impl_report!(speciation(&mut self, event: Unused) -> MaybeUsed<ReportSpeciation> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.filter.report_speciation {
                    plugin.reporter.report_speciation(Unused::new(event));
                }
            }
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> MaybeUsed<ReportDispersal> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.filter.report_dispersal {
                    plugin.reporter.report_dispersal(Unused::new(event));
                }
            }
        })
    });

    impl_report!(progress(&mut self, event: Unused) -> MaybeUsed<ReportProgress> {
        event.maybe_use_in(|event| {
            for plugin in self.plugins.iter_mut() {
                if plugin.filter.report_progress {
                    plugin.reporter.report_progress(Unused::new(event));
                }
            }
        })
    });

    impl_finalise!((self) {
        for plugin in self.plugins.into_vec() {
            plugin.finalise()
        }
    });

    fn initialise(&mut self) -> Result<(), String> {
        for plugin in self.plugins.iter_mut() {
            plugin.reporter.initialise()?;
        }

        Ok(())
    }
}

#[allow(clippy::pub_enum_variant_names)]
#[derive(Debug)]
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

        let report_speciation = plugins
            .iter()
            .any(|reporter| reporter.filter.report_speciation);
        let report_dispersal = plugins
            .iter()
            .any(|reporter| reporter.filter.report_dispersal);
        let report_progress = plugins
            .iter()
            .any(|reporter| reporter.filter.report_progress);

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

#[macro_export]
macro_rules! match_any_reporter_plugin_vec {
    ($any:expr => | $inner:ident | $code:block) => {{
        use $crate::import::AnyReporterPluginVec::*;

        match $any {
            IgnoreSpeciationIgnoreDispersalIgnoreProgress($inner) => $code,
            IgnoreSpeciationIgnoreDispersalReportProgress($inner) => $code,
            IgnoreSpeciationReportDispersalIgnoreProgress($inner) => $code,
            IgnoreSpeciationReportDispersalReportProgress($inner) => $code,
            ReportSpeciationIgnoreDispersalIgnoreProgress($inner) => $code,
            ReportSpeciationIgnoreDispersalReportProgress($inner) => $code,
            ReportSpeciationReportDispersalIgnoreProgress($inner) => $code,
            ReportSpeciationReportDispersalReportProgress($inner) => $code,
        }
    }};
    ($any:expr => | mut $inner:ident | $code:block) => {{
        use $crate::import::AnyReporterPluginVec::*;

        match $any {
            IgnoreSpeciationIgnoreDispersalIgnoreProgress(mut $inner) => $code,
            IgnoreSpeciationIgnoreDispersalReportProgress(mut $inner) => $code,
            IgnoreSpeciationReportDispersalIgnoreProgress(mut $inner) => $code,
            IgnoreSpeciationReportDispersalReportProgress(mut $inner) => $code,
            ReportSpeciationIgnoreDispersalIgnoreProgress(mut $inner) => $code,
            ReportSpeciationIgnoreDispersalReportProgress(mut $inner) => $code,
            ReportSpeciationReportDispersalIgnoreProgress(mut $inner) => $code,
            ReportSpeciationReportDispersalReportProgress(mut $inner) => $code,
        }
    }};
}

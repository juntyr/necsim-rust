use necsim_core::reporter::{boolean::Boolean, FilteredReporter, Reporter};

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext: core::fmt::Debug {
    type Reporter: Reporter;

    /// # Errors
    ///
    /// Return any error which occured while building the reporter(s)
    fn try_build<KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean>(
        self,
    ) -> anyhow::Result<FilteredReporter<Self::Reporter, KeepSpeciation, KeepDispersal, KeepProgress>>;
}

#[allow(clippy::module_name_repetitions)]
pub trait FinalisableReporter {
    fn finalise(self);
}

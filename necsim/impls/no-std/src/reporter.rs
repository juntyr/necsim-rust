use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext: core::fmt::Debug {
    type Reporter: Reporter;

    /// # Errors
    ///
    /// Return any error which occured while building the reporter(s)
    fn try_build(self) -> anyhow::Result<Self::Reporter>;
}

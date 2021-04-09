use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext: core::fmt::Debug {
    type Reporter: Reporter;

    fn build(self) -> Self::Reporter;
}

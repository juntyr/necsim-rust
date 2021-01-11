use necsim_core::reporter::Reporter;

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext {
    type Reporter: Reporter;

    fn with_reporter<O, F: FnOnce(&mut Self::Reporter) -> O>(self, inner: F) -> O;
}

use necsim_core::{
    cogs::{Habitat, LineageReference},
    reporter::Reporter,
};

#[allow(clippy::module_name_repetitions)]
pub trait ReporterContext {
    type Reporter<H: Habitat, R: LineageReference<H>>: Reporter<H, R>;

    fn with_reporter<
        O,
        H: Habitat,
        R: LineageReference<H>,
        F: FnOnce(&mut Self::Reporter<H, R>) -> O,
    >(
        self,
        inner: F,
    ) -> O;
}

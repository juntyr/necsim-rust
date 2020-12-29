use crate::{cogs::Habitat, landscape::Location};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait SpeciationProbability<H: Habitat>: core::fmt::Debug {
    #[must_use]
    #[debug_ensures((0.0_f64..=1.0_f64).contains(&ret), "returns a probability")]
    fn get_speciation_probability_at_location(&self, location: &Location) -> f64;
}

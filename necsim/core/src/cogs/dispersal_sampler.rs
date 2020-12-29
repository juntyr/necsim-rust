use crate::{cogs::RngCore, landscape::Location};

use super::Habitat;

pub trait DispersalSampler<H: Habitat, G: RngCore>: core::fmt::Debug {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut G) -> Location;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait SeparableDispersalSampler<H: Habitat, G: RngCore>: DispersalSampler<H, G> {
    #[must_use]
    #[debug_ensures(&ret != location, "disperses to a different location")]
    fn sample_non_self_dispersal_from_location(&self, location: &Location, rng: &mut G)
        -> Location;

    #[must_use]
    #[debug_ensures((0.0_f64..=1.0_f64).contains(&ret), "returns probability")]
    fn get_self_dispersal_probability_at_location(&self, location: &Location) -> f64;
}
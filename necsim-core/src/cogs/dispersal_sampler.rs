use crate::landscape::Location;
use crate::rng::Rng;

use super::Habitat;

pub trait DispersalSampler<H: Habitat>: core::fmt::Debug {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait SeparableDispersalSampler<H: Habitat>: DispersalSampler<H> {
    #[must_use]
    #[debug_ensures(&ret != location, "disperses to a different location")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        rng: &mut impl Rng,
    ) -> Location;

    #[must_use]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "returns probability")]
    fn get_self_dispersal_probability_at_location(&self, location: &Location) -> f64;
}

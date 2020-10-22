use crate::landscape::Location;
use crate::rng::Rng;

use super::{Habitat, LineageReference, LineageSampler};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<H: Habitat, R: LineageReference<H>, L: LineageSampler<H, R>> {
    #[must_use]
    #[debug_requires(habitat > 0, "location is habitable")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<R>;
}

// TODO: Move ConditionalCoalescenceSampler into separate crate as it is not core
#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ConditionalCoalescenceSampler<H: Habitat, R: LineageReference<H>, L: LineageSampler<H, R>>:
    CoalescenceSampler<H, R, L>
{
    #[must_use]
    fn sample_coalescence_at_location(&self, location: &Location, rng: &mut impl Rng) -> R;

    #[must_use]
    #[debug_requires(habitat > 0, "location is habitable")]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "returns probability")]
    fn get_coalescence_probability_at_location(&self, location: &Location, habitat: u32) -> f64;
}

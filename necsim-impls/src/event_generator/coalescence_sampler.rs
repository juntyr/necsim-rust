use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use necsim_core::lineage::LineageReference;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<L: LineageReference> {
    #[must_use]
    #[debug_requires(habitat > 0, "location is habitable")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<L>;
}

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ConditionalCoalescenceSampler<L: LineageReference>: CoalescenceSampler<L> {
    #[must_use]
    fn sample_coalescence_at_location(&self, location: &Location, rng: &mut impl Rng) -> L;

    #[must_use]
    #[debug_requires(habitat > 0, "location is habitable")]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "returns probability")]
    fn get_coalescence_probability_at_location(&self, location: &Location, habitat: u32) -> f64;
}

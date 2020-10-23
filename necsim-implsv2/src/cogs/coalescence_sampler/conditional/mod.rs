use necsim_corev2::cogs::{CoalescenceSampler, Habitat, LineageReference, LineageStore};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;

pub mod r#impl;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ConditionalCoalescenceSampler<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>:
    CoalescenceSampler<H, R, S>
{
    #[must_use]
    fn sample_coalescence_at_location(
        &self,
        location: &Location,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> R;

    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(location) > 0, "location is habitable")]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "returns probability")]
    fn get_coalescence_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        lineage_store_includes_self: bool,
    ) -> f64;
}

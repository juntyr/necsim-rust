use necsim_corev2::cogs::{CoalescenceSampler, Habitat, LineageReference, LineageStore};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;

use super::super::optional_coalescence;
use super::ConditionalCoalescenceSampler as ConditionalCoalescenceSamplerTrait;

pub struct ConditionalCoalescenceSampler;

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> CoalescenceSampler<H, R, S>
    for ConditionalCoalescenceSampler
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> Option<R> {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            rng,
        )
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>
    ConditionalCoalescenceSamplerTrait<H, R, S> for ConditionalCoalescenceSampler
{
    #[must_use]
    fn sample_coalescence_at_location(
        &self,
        location: &Location,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> R {
        let lineages_at_location = lineage_store.get_active_lineages_at_location(location);
        let population = lineages_at_location.len();

        let chosen_coalescence = rng.sample_index(population);

        lineages_at_location[chosen_coalescence].clone()
    }

    #[must_use]
    fn get_coalescence_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        lineage_store_includes_self: bool,
    ) -> f64 {
        // If the lineage store includes self, the population must be decremented
        // to avoid coalescence with the self lineage

        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_active_lineages_at_location(location)
            .len()
            - usize::from(lineage_store_includes_self)) as f64;
        let habitat = f64::from(habitat.get_habitat_at_location(location));

        population / habitat
    }
}

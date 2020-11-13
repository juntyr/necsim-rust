use core::marker::PhantomData;

use necsim_core::{
    cogs::{CoalescenceSampler, CoherentLineageStore, Habitat, LineageReference, RngCore},
    landscape::{IndexedLocation, Location},
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ConditionalCoalescenceSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(PhantomData<(H, G, R, S)>);

impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: CoherentLineageStore<H, R>> Default
    for ConditionalCoalescenceSampler<H, G, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: CoherentLineageStore<H, R>>
    CoalescenceSampler<H, G, R, S> for ConditionalCoalescenceSampler<H, G, R, S>
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut G,
    ) -> (IndexedLocation, Option<R>) {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            rng,
        )
    }
}

impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: CoherentLineageStore<H, R>>
    ConditionalCoalescenceSampler<H, G, R, S>
{
    #[must_use]
    pub fn sample_coalescence_at_location(
        location: Location,
        lineage_store: &S,
        rng: &mut G,
    ) -> (IndexedLocation, R) {
        use necsim_core::cogs::RngSampler;

        let lineages_at_location = lineage_store.get_active_lineages_at_location(&location);
        let population = lineages_at_location.len();

        let chosen_coalescence_index = rng.sample_index(population);

        #[allow(clippy::cast_possible_truncation)]
        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index as u32);
        let chosen_coalescence = lineages_at_location[chosen_coalescence_index].clone();

        (indexed_location, chosen_coalescence)
    }

    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(location) > 0, "location is habitable")]
    #[debug_ensures((0.0_f64..=1.0_f64).contains(&ret), "returns probability")]
    pub fn get_coalescence_probability_at_location(
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

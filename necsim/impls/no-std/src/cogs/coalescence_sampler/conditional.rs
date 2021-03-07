use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoalescenceSampler, CoherentLineageStore, Habitat,
        LineageReference,
    },
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ConditionalCoalescenceSampler<
    H: Habitat,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(PhantomData<(H, R, S)>);

impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>> Default
    for ConditionalCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>> Backup
    for ConditionalCoalescenceSampler<H, R, S>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>> CoalescenceSampler<H, R, S>
    for ConditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, Option<GlobalLineageReference>) {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            coalescence_rng_sample,
        )
    }
}

impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>>
    ConditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    pub fn sample_coalescence_at_location(
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, GlobalLineageReference) {
        let lineages_at_location = lineage_store
            .get_active_local_lineage_references_at_location_unordered(&location, habitat);

        #[allow(clippy::cast_possible_truncation)]
        let population = lineages_at_location.len() as u32;

        let chosen_coalescence_index = coalescence_rng_sample.sample_coalescence_index(population);

        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

        let chosen_coalescence = lineages_at_location[chosen_coalescence_index as usize].clone();

        (
            indexed_location,
            lineage_store[chosen_coalescence].global_reference().clone(),
        )
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
        //  to avoid coalescence with the currently active lineage

        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_active_local_lineage_references_at_location_unordered(location, habitat)
            .len()
            - usize::from(lineage_store_includes_self)) as f64;
        let habitat = f64::from(habitat.get_habitat_at_location(location));

        population / habitat
    }
}

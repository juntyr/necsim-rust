use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, F64Core,
        GloballyCoherentLineageStore, Habitat, LineageReference,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, LineageInteraction},
};
use necsim_core_bond::ClosedUnitF64;

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ConditionalCoalescenceSampler<
    F: F64Core,
    H: Habitat<F>,
    R: LineageReference<F, H>,
    S: GloballyCoherentLineageStore<F, H, R>,
>(PhantomData<(F, H, R, S)>);

impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
    > Default for ConditionalCoalescenceSampler<F, H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(F, H, R, S)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
    > Backup for ConditionalCoalescenceSampler<F, H, R, S>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(F, H, R, S)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
    > CoalescenceSampler<F, H, R, S> for ConditionalCoalescenceSampler<F, H, R, S>
{
    #[must_use]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        optional_coalescence::sample_interaction_at_location(
            location,
            habitat,
            lineage_store,
            coalescence_rng_sample,
        )
    }
}

impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
    > ConditionalCoalescenceSampler<F, H, R, S>
{
    #[must_use]
    pub fn sample_coalescence_at_location(
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, GlobalLineageReference) {
        let lineages_at_location =
            lineage_store.get_local_lineage_references_at_location_unordered(&location, habitat);

        #[allow(clippy::cast_possible_truncation)]
        let population = lineages_at_location.len() as u32;

        let chosen_coalescence_index =
            coalescence_rng_sample.sample_coalescence_index::<F>(population);
        let chosen_coalescence = lineages_at_location[chosen_coalescence_index as usize].clone();

        let lineage = &lineage_store[chosen_coalescence];

        let indexed_location = IndexedLocation::new(location, lineage.indexed_location.index());

        (indexed_location, lineage.global_reference.clone())
    }

    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(location) > 0, "location is habitable")]
    pub fn get_coalescence_probability_at_location(
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        lineage_store_includes_self: bool,
    ) -> ClosedUnitF64 {
        // If the lineage store includes self, the population must be decremented
        //  to avoid coalescence with the currently active lineage

        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_local_lineage_references_at_location_unordered(location, habitat)
            .len()
            - usize::from(lineage_store_includes_self)) as f64;
        let habitat = f64::from(habitat.get_habitat_at_location(location));

        // Safety: Normalised probability in range [0.0; 1.0]
        unsafe { ClosedUnitF64::new_unchecked(population / habitat) }
    }
}

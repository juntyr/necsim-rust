use core::num::NonZeroU32;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Habitat, LocallyCoherentLineageStore, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

#[must_use]
pub fn sample_interaction_at_location<
    M: MathsCore,
    H: Habitat<M>,
    S: LocallyCoherentLineageStore<M, H>,
>(
    location: Location,
    habitat: &H,
    lineage_store: &S,
    coalescence_rng_sample: CoalescenceRngSample,
) -> (IndexedLocation, LineageInteraction) {
    // Safety: individuals can only occupy habitable locations
    let habitat_at_location =
        unsafe { NonZeroU32::new_unchecked(habitat.get_habitat_at_location(&location)) };

    let chosen_coalescence_index =
        coalescence_rng_sample.sample_coalescence_index(habitat_at_location);

    let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

    let optional_coalescence = lineage_store
        .get_global_lineage_reference_at_indexed_location(&indexed_location, habitat)
        .cloned();

    (indexed_location, optional_coalescence.into())
}

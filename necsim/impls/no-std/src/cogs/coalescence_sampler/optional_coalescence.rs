use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Habitat, LineageReference,
        LocallyCoherentLineageStore, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

#[must_use]
pub fn sample_interaction_at_location<
    M: MathsCore,
    H: Habitat<M>,
    R: LineageReference<M, H>,
    S: LocallyCoherentLineageStore<M, H, R>,
>(
    location: Location,
    habitat: &H,
    lineage_store: &S,
    coalescence_rng_sample: CoalescenceRngSample,
) -> (IndexedLocation, LineageInteraction) {
    let chosen_coalescence_index = coalescence_rng_sample
        .sample_coalescence_index::<M>(habitat.get_habitat_at_location(&location));

    let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

    let optional_coalescence = lineage_store
        .get_global_lineage_reference_at_indexed_location(&indexed_location, habitat)
        .cloned();

    (indexed_location, optional_coalescence.into())
}

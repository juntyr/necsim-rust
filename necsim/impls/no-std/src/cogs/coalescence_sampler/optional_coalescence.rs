use necsim_core::{
    cogs::{CoalescenceRngSample, Habitat, LineageReference, LocallyCoherentLineageStore},
    event::LineageInteraction,
    landscape::{IndexedLocation, Location},
};

#[must_use]
pub fn sample_interaction_at_location<
    H: Habitat,
    R: LineageReference<H>,
    S: LocallyCoherentLineageStore<H, R>,
>(
    location: Location,
    habitat: &H,
    lineage_store: &S,
    coalescence_rng_sample: CoalescenceRngSample,
) -> (IndexedLocation, LineageInteraction) {
    let chosen_coalescence_index =
        coalescence_rng_sample.sample_coalescence_index(habitat.get_habitat_at_location(&location));

    let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

    let optional_coalescence = lineage_store
        .get_active_global_lineage_reference_at_indexed_location(&indexed_location, habitat)
        .cloned();

    (indexed_location, optional_coalescence.into())
}

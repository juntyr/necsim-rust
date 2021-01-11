use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, RngCore},
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

#[must_use]
pub fn sample_optional_coalescence_at_location<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(
    location: Location,
    habitat: &H,
    lineage_store: &S,
    rng: &mut G,
) -> (IndexedLocation, Option<GlobalLineageReference>) {
    use necsim_core::cogs::RngSampler;

    let chosen_coalescence_index = rng.sample_index_u32(habitat.get_habitat_at_location(&location));

    let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

    let optional_coalescence = lineage_store
        .get_active_global_lineage_reference_at_indexed_location(&indexed_location)
        .cloned();

    (indexed_location, optional_coalescence)
}

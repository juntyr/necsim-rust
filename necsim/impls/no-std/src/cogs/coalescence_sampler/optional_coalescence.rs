use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, RngCore},
    landscape::{IndexedLocation, Location},
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
) -> (IndexedLocation, Option<R>) {
    use necsim_core::cogs::RngSampler;

    let lineages_at_location = lineage_store.get_active_lineages_at_location(&location);

    let chosen_coalescence_index =
        rng.sample_index(habitat.get_habitat_at_location(&location) as usize);

    #[allow(clippy::cast_possible_truncation)]
    // As lineages are stored in a continuous slice, there are two cases:
    //  a) coalescence:    the dispersal target location index equals the index of the parent
    //  b) no coalescence: the dispersal target location index is clamped to |lineages_at_location|
    //                     as the lineage will be appended to the continuous slice of lineages at
    //                     the location, lineages_at_location
    let indexed_location = IndexedLocation::new(
        location,
        (chosen_coalescence_index as u32).min(lineages_at_location.len() as u32),
    );
    let optional_coalescence = lineages_at_location.get(chosen_coalescence_index).cloned();

    (indexed_location, optional_coalescence)
}

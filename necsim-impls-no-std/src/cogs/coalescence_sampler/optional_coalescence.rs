use necsim_core::cogs::{CoherentLineageStore, Habitat, LineageReference, RngCore};
use necsim_core::landscape::Location;

#[must_use]
pub fn sample_optional_coalescence_at_location<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(
    location: &Location,
    habitat: &H,
    lineage_store: &S,
    rng: &mut G,
) -> Option<R> {
    use necsim_core::cogs::RngSampler;

    let lineages_at_location = lineage_store.get_active_lineages_at_location(location);
    let population = lineages_at_location.len();

    let chosen_coalescence = rng.sample_index(habitat.get_habitat_at_location(location) as usize);

    if chosen_coalescence >= population {
        return None;
    }

    Some(lineages_at_location[chosen_coalescence].clone())
}

use necsim_core::cogs::{Habitat, LineageReference, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

#[must_use]
pub fn sample_optional_coalescence_at_location<
    H: Habitat,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
>(
    location: &Location,
    habitat: &H,
    lineage_store: &S,
    rng: &mut impl Rng,
) -> Option<R> {
    let lineages_at_location = lineage_store.get_active_lineages_at_location(location);
    let population = lineages_at_location.len();

    let chosen_coalescence = rng.sample_index(habitat.get_habitat_at_location(location) as usize);

    if chosen_coalescence >= population {
        return None;
    }

    Some(lineages_at_location[chosen_coalescence].clone())
}

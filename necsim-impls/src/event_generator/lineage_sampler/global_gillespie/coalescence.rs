use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{GlobalGillespieStore, LineageReference};

impl GlobalGillespieStore {
    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    #[debug_requires(habitat > 0, "location is habitable")]
    #[debug_ensures(
        ret.is_some() -> self.explicit_global_store_lineage_at_location_contract(ret.unwrap()),
        "lineage is at the location and index it references"
    )]
    pub fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        let population = self.get_number_active_lineages_at_location(location);

        let chosen_coalescence = rng.sample_index(habitat as usize);

        if chosen_coalescence >= population {
            return None;
        }

        Some(
            self.location_to_lineage_references[(
                (location.y() - self.landscape_extent.y()) as usize,
                (location.x() - self.landscape_extent.x()) as usize,
            )][chosen_coalescence],
        )
    }
}

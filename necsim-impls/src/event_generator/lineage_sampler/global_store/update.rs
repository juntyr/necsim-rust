use necsim_core::landscape::Location;

use super::{GlobalLineageStore, LineageReference};

impl GlobalLineageStore {
    #[debug_requires(reference.0 < self.lineages_store.len(), "lineage reference is in range")]
    #[debug_requires(
        self.landscape_extent.contains(&location),
        "location is inside landscape extent"
    )]
    #[debug_requires(
        !self.explicit_global_store_lineage_at_location_contract(reference),
        "lineage is not at the location and index it references"
    )]
    #[debug_requires(
        self.explicit_global_store_invariant_contract(&location),
        "invariant of lineage-location bijection holds"
    )]
    #[debug_ensures(
        self[reference].location() == &old(location.clone()),
        "lineage was added to location"
    )]
    #[debug_ensures(
        self.explicit_global_store_lineage_at_location_contract(reference),
        "lineage is at the location and index it references"
    )]
    #[debug_ensures(
        self.explicit_global_store_invariant_contract(&old(location.clone())),
        "maintains invariant of lineage-location bijection"
    )]
    pub fn add_lineage_to_location(&mut self, reference: LineageReference, location: Location) {
        let lineages_at_location = &mut self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )];

        // TODO: We should be able to assert that we never surpass the available habitat

        lineages_at_location.push(reference);

        unsafe {
            self.lineages_store[reference.0]
                .move_to_location(location, lineages_at_location.len() - 1)
        };
    }

    #[debug_requires(reference.0 < self.lineages_store.len(), "lineage reference is in range")]
    #[debug_requires(
        self.landscape_extent.contains(self[reference].location()),
        "lineage's location is inside landscape extent"
    )]
    #[debug_requires(
        self.explicit_global_store_lineage_at_location_contract(reference),
        "lineage is at the location and index it references"
    )]
    #[debug_requires(
        self.explicit_global_store_invariant_contract(self[reference].location()),
        "invariant of lineage-location bijection holds"
    )]
    #[debug_ensures(
        !self.explicit_global_store_lineage_at_location_contract(reference),
        "lineage was removed from the location and index it references"
    )]
    #[debug_ensures(
        self.explicit_global_store_invariant_contract(self[reference].location()),
        "maintains invariant of lineage-location bijection"
    )]
    pub fn remove_lineage_from_its_location(&mut self, reference: LineageReference) {
        let lineage = &self.lineages_store[reference.0];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (lineage.location().y() - self.landscape_extent.y()) as usize,
            (lineage.location().x() - self.landscape_extent.x()) as usize,
        )];

        if let Some(last_lineage_at_location) = lineages_at_location.pop() {
            let lineage_index_at_location = lineage.index_at_location();

            if lineage_index_at_location < lineages_at_location.len() {
                lineages_at_location[lineage_index_at_location] = last_lineage_at_location;

                unsafe {
                    self.lineages_store[last_lineage_at_location.0]
                        .update_index_at_location(lineage_index_at_location)
                };
            }
        }
    }
}

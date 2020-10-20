use necsim_core::landscape::Location;

use super::{GlobalLineageStore, LineageReference};

impl GlobalLineageStore {
    #[must_use]
    pub fn explicit_global_store_lineage_at_location_contract(
        &self,
        reference: LineageReference,
    ) -> bool {
        if reference.0 >= self.lineages_store.len() {
            return false;
        }

        let lineage = &self[reference];

        let lineages_at_location = &self.location_to_lineage_references[(
            (lineage.location().y() - self.landscape_extent.y()) as usize,
            (lineage.location().x() - self.landscape_extent.x()) as usize,
        )];

        match lineages_at_location.get(lineage.index_at_location()) {
            Some(reference_at_location) => reference_at_location == &reference,
            None => false,
        }
    }

    #[must_use]
    pub(super) fn explicit_global_store_invariant_contract(&self, location: &Location) -> bool {
        let lineages_at_location = &self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )];

        lineages_at_location
            .iter()
            .enumerate()
            .all(|(i, reference)| {
                self[*reference].location() == location && self[*reference].index_at_location() == i
            })
    }
}

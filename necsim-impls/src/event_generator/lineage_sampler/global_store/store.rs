use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{GlobalLineageStore, LineageReference};

impl GlobalLineageStore {
    #[must_use]
    #[debug_ensures(match ret {
        Some(_) => self.number_active_lineages() == old(self.number_active_lineages()) - 1,
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> !self.explicit_global_store_lineage_at_location_contract(ret.unwrap()),
        "lineage was removed from the location and index it references"
    )]
    pub fn pop_random_active_lineage_reference(
        &mut self,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        let last_active_lineage_reference = match self.active_lineage_references.pop() {
            Some(reference) => reference,
            None => return None,
        };

        let chosen_active_lineage_index =
            rng.sample_index(self.active_lineage_references.len() + 1);

        let chosen_lineage_reference =
            if chosen_active_lineage_index == self.active_lineage_references.len() {
                last_active_lineage_reference
            } else {
                let chosen_lineage_reference =
                    self.active_lineage_references[chosen_active_lineage_index];

                self.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                chosen_lineage_reference
            };

        self.remove_lineage_from_its_location(chosen_lineage_reference);

        Some(chosen_lineage_reference)
    }

    #[debug_requires(
        !self.explicit_global_store_lineage_at_location_contract(reference),
        "lineage is not at the location and index it references"
    )]
    #[debug_ensures(
        self[reference].location() == &old(location.clone()),
        "lineage was added to location"
    )]
    #[debug_ensures(
        self.number_active_lineages() == old(self.number_active_lineages()) + 1,
        "an active lineage was added"
    )]
    pub fn push_active_lineage_reference_at_location(
        &mut self,
        reference: LineageReference,
        location: Location,
    ) {
        self.add_lineage_to_location(reference, location);

        self.active_lineage_references.push(reference);
    }
}

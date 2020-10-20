use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::{EventTime, GlobalGillespieStore, LineageReference};

impl GlobalGillespieStore {
    #[must_use]
    #[debug_ensures(match ret {
        Some(_) => self.number_active_lineages() == old(self.number_active_lineages()) - 1,
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> !self.explicit_global_store_lineage_at_location_contract(ret.unwrap().0),
        "lineage was removed from the location and index it references"
    )]
    pub fn pop_random_active_lineage_reference_and_event_time(
        &mut self,
        rng: &mut impl Rng,
    ) -> Option<(LineageReference, f64)> {
        let (chosen_active_location, chosen_event_time) = match self.active_locations.pop() {
            Some(val) => val,
            None => return None,
        };

        let location_index = (
            (chosen_active_location.y() - self.landscape_extent.y()) as usize,
            (chosen_active_location.x() - self.landscape_extent.x()) as usize,
        );

        let lineages_at_location = &self.location_to_lineage_references[location_index];

        let chosen_lineage_index_at_location = rng.sample_index(lineages_at_location.len());
        let chosen_lineage_reference = lineages_at_location[chosen_lineage_index_at_location];

        self.remove_lineage_from_its_location(chosen_lineage_reference);
        self.number_active_lineages -= 1;

        let number_lineages_at_location = self.location_to_lineage_references[location_index].len();

        if number_lineages_at_location > 0 {
            #[allow(clippy::cast_precision_loss)]
            let lambda = 0.5_f64 * number_lineages_at_location as f64;

            self.active_locations.push(
                chosen_active_location,
                EventTime(chosen_event_time.0 + rng.sample_exponential(lambda)),
            );
        }

        Some((chosen_lineage_reference, chosen_event_time.0))
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
        time: f64,
        rng: &mut impl Rng,
    ) {
        self.add_lineage_to_location(reference, location.clone());

        let number_lineages_at_location = self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )]
            .len();

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * number_lineages_at_location as f64;

        self.active_locations
            .push(location, EventTime(time + rng.sample_exponential(lambda)));

        self.number_active_lineages += 1;
    }
}

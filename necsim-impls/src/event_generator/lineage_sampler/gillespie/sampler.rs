use float_next_after::NextAfter;

use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use crate::event_generator::lineage_sampler::global_store::LineageReference;

use crate::event_generator::lineage_sampler::LineageSampler;

use super::{EventTime, GillespieLineageSampler};

#[contract_trait]
impl LineageSampler<LineageReference> for GillespieLineageSampler {
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    #[debug_ensures(
        ret.is_some() -> !self.lineages_store.explicit_global_store_lineage_at_location_contract(ret.unwrap().0),
        "lineage was removed from the location and index it references"
    )]
    fn pop_next_active_lineage_reference_and_event_time(
        &mut self,
        time: f64,
        rng: &mut impl Rng,
    ) -> Option<(LineageReference, f64)> {
        let (chosen_active_location, chosen_event_time) = match self.active_locations.pop() {
            Some(val) => val,
            None => return None,
        };

        let lineages_at_location = self
            .lineages_store
            .get_active_lineages_at_location(&chosen_active_location);
        let number_lineages_at_location = lineages_at_location.len() - 1;

        let chosen_lineage_index_at_location = rng.sample_index(lineages_at_location.len());
        let chosen_lineage_reference = lineages_at_location[chosen_lineage_index_at_location];

        self.lineages_store
            .remove_lineage_from_its_location(chosen_lineage_reference);
        self.number_active_lineages -= 1;

        if number_lineages_at_location > 0 {
            #[allow(clippy::cast_precision_loss)]
            let lambda = 0.5_f64 * number_lineages_at_location as f64;

            self.active_locations.push(
                chosen_active_location,
                EventTime(chosen_event_time.0 + rng.sample_exponential(lambda)),
            );
        }

        let unique_event_time: f64 = if chosen_event_time.0 > time {
            chosen_event_time.0
        } else {
            // println!("Event time not increased: {} {}", time, chosen_event_time.0);

            time.next_after(f64::INFINITY)
        };

        Some((chosen_lineage_reference, unique_event_time))
    }

    #[debug_requires(
        !self.lineages_store.explicit_global_store_lineage_at_location_contract(reference),
        "lineage is not at the location and index it references"
    )]
    fn add_lineage_reference_to_location(
        &mut self,
        reference: LineageReference,
        location: Location,
        time: f64,
        rng: &mut impl Rng,
    ) {
        self.lineages_store
            .add_lineage_to_location(reference, location.clone());

        let number_lineages_at_location = self
            .lineages_store
            .get_active_lineages_at_location(&location)
            .len();

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * number_lineages_at_location as f64;

        self.active_locations
            .push(location, EventTime(time + rng.sample_exponential(lambda)));

        self.number_active_lineages += 1;
    }
}

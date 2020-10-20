use float_next_after::NextAfter;

use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use crate::event_generator::lineage_sampler::global_store::LineageReference;

use crate::event_generator::lineage_sampler::LineageSampler;

use super::ActiveLineageListSampler;

#[contract_trait]
impl LineageSampler<LineageReference> for ActiveLineageListSampler {
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
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

        self.lineages_store
            .remove_lineage_from_its_location(chosen_lineage_reference);

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * (self.number_active_lineages() + 1) as f64;

        let event_time = time + rng.sample_exponential(lambda);

        let unique_event_time: f64 = if event_time > time {
            event_time
        } else {
            event_time.next_after(f64::INFINITY)
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
        _time: f64,
        _rng: &mut impl Rng,
    ) {
        self.lineages_store
            .add_lineage_to_location(reference, location);

        self.active_lineage_references.push(reference);
    }
}

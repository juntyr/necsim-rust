use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, CoherentLineageStore, DispersalSampler, Habitat, LineageReference,
    RngCore,
};
use necsim_core::landscape::IndexedLocation;
use necsim_core::simulation::partial::active_lineager_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::unconditional::UnconditionalCoalescenceSampler;
use crate::cogs::event_sampler::unconditional::UnconditionalEventSampler;

use super::ClassicalActiveLineageSampler;

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    >
    ActiveLineageSampler<
        H,
        G,
        D,
        R,
        S,
        UnconditionalCoalescenceSampler<H, G, R, S>,
        UnconditionalEventSampler<H, G, D, R, S, UnconditionalCoalescenceSampler<H, G, R, S>>,
    > for ClassicalActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    fn get_time_of_last_event(&self, _lineage_store: &S) -> f64 {
        self.last_event_time
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler<H, G, R, S>,
            UnconditionalEventSampler<H, G, D, R, S, UnconditionalCoalescenceSampler<H, G, R, S>>,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        use necsim_core::cogs::RngSampler;

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
                    self.active_lineage_references[chosen_active_lineage_index].clone();

                self.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                chosen_lineage_reference
            };

        let lineage_indexed_location = simulation
            .lineage_store
            .pop_lineage_from_its_location(chosen_lineage_reference.clone());

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * ((self.number_active_lineages() + 1) as f64);

        let event_time = time + rng.sample_exponential(lambda);

        let unique_event_time: f64 = if event_time > time {
            event_time
        } else {
            time.next_after(f64::INFINITY)
        };

        simulation
            .lineage_store
            .update_lineage_time_of_last_event(chosen_lineage_reference.clone(), unique_event_time);

        self.last_event_time = unique_event_time;

        Some((
            chosen_lineage_reference,
            lineage_indexed_location,
            unique_event_time,
        ))
    }

    #[allow(clippy::type_complexity)]
    #[debug_requires(
        indexed_location.index() as usize ==
            simulation.lineage_store.get_active_lineages_at_location(
                indexed_location.location()
            ).len(),
        "location index equals the append index at the location"
    )]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        _time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler<H, G, R, S>,
            UnconditionalEventSampler<H, G, D, R, S, UnconditionalCoalescenceSampler<H, G, R, S>>,
        >,
        _rng: &mut G,
    ) {
        simulation
            .lineage_store
            .append_lineage_to_location(lineage_reference.clone(), indexed_location.into());

        self.active_lineage_references.push(lineage_reference);
    }
}

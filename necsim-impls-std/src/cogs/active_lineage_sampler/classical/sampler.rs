use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, DispersalSampler, Habitat, LineageReference, LineageStore,
};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::active_lineager_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::unconditional::UnconditionalCoalescenceSampler;
use crate::cogs::event_sampler::unconditional::UnconditionalEventSampler;

use super::ClassicalActiveLineageSampler;

#[contract_trait]
impl<H: Habitat, D: DispersalSampler<H>, R: LineageReference<H>, S: LineageStore<H, R>>
    ActiveLineageSampler<
        H,
        D,
        R,
        S,
        UnconditionalCoalescenceSampler<H, R, S>,
        UnconditionalEventSampler<H, D, R, S, UnconditionalCoalescenceSampler<H, R, S>>,
    > for ClassicalActiveLineageSampler<H, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_and_time_of_next_event(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler<H, R, S>,
            UnconditionalEventSampler<H, D, R, S, UnconditionalCoalescenceSampler<H, R, S>>,
        >,
        rng: &mut impl Rng,
    ) -> Option<(R, f64)> {
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

        simulation
            .lineage_store
            .remove_lineage_from_its_location(chosen_lineage_reference.clone());

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * ((self.number_active_lineages() + 1) as f64);

        let event_time = time + rng.sample_exponential(lambda);

        let unique_event_time: f64 = if event_time > time {
            event_time
        } else {
            time.next_after(f64::INFINITY)
        };

        Some((chosen_lineage_reference, unique_event_time))
    }

    #[allow(clippy::type_complexity)]
    fn push_active_lineage_to_location(
        &mut self,
        lineage_reference: R,
        location: Location,
        _time: f64,
        simulation: &mut PartialSimulation<
            H,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler<H, R, S>,
            UnconditionalEventSampler<H, D, R, S, UnconditionalCoalescenceSampler<H, R, S>>,
        >,
        _rng: &mut impl Rng,
    ) {
        simulation
            .lineage_store
            .add_lineage_to_location(lineage_reference.clone(), location);

        self.active_lineage_references.push(lineage_reference);
    }
}

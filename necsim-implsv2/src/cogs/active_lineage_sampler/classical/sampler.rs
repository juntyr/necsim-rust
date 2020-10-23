use float_next_after::NextAfter;

use necsim_corev2::cogs::{
    ActiveLineageSampler, DispersalSampler, Habitat, LineageReference, LineageStore,
};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;
use necsim_corev2::simulation::Simulation;

use crate::cogs::coalescence_sampler::unconditional::UnconditionalCoalescenceSampler;
use crate::cogs::event_sampler::unconditional::UnconditionalEventSampler;

use super::ClassicalActiveLineageSampler;

#[contract_trait]
impl<H: Habitat, D: DispersalSampler<H>, R: LineageReference<H>, S: LineageStore<H, R>>
    ActiveLineageSampler<H, D, R, S, UnconditionalCoalescenceSampler, UnconditionalEventSampler>
    for ClassicalActiveLineageSampler<H, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    #[must_use]
    fn pop_active_lineage_and_time_of_next_event(
        time: f64,
        simulation: &mut Simulation<
            H,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler,
            UnconditionalEventSampler,
            Self,
        >,
        rng: &mut impl Rng,
    ) -> Option<(R, f64)> {
        let this = simulation.active_lineage_sampler_mut();

        let last_active_lineage_reference = match this.active_lineage_references.pop() {
            Some(reference) => reference,
            None => return None,
        };

        let chosen_active_lineage_index =
            rng.sample_index(this.active_lineage_references.len() + 1);

        let chosen_lineage_reference =
            if chosen_active_lineage_index == this.active_lineage_references.len() {
                last_active_lineage_reference
            } else {
                let chosen_lineage_reference =
                    this.active_lineage_references[chosen_active_lineage_index].clone();

                this.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                chosen_lineage_reference
            };

        simulation
            .lineage_store_mut()
            .remove_lineage_from_its_location(chosen_lineage_reference.clone());

        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64
            * (simulation
                .active_lineage_sampler_mut()
                .number_active_lineages() as f64);

        let event_time = time + rng.sample_exponential(lambda);

        let unique_event_time: f64 = if event_time > time {
            event_time
        } else {
            event_time.next_after(f64::INFINITY)
        };

        Some((chosen_lineage_reference, unique_event_time))
    }

    fn push_active_lineage_to_location(
        lineage_reference: R,
        location: Location,
        _time: f64,
        simulation: &mut Simulation<
            H,
            D,
            R,
            S,
            UnconditionalCoalescenceSampler,
            UnconditionalEventSampler,
            Self,
        >,
        _rng: &mut impl Rng,
    ) {
        simulation
            .lineage_store_mut()
            .add_lineage_to_location(lineage_reference.clone(), location);

        simulation
            .active_lineage_sampler_mut()
            .active_lineage_references
            .push(lineage_reference);
    }
}

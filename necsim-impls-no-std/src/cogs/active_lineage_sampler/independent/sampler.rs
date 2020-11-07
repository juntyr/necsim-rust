use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, DispersalSampler, Habitat, IncoherentLineageStore, LineageReference,
};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::active_lineager_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;
use crate::cogs::event_sampler::independent::IndependentEventSampler;

use super::IndependentActiveLineageSampler;

#[contract_trait]
impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    ActiveLineageSampler<
        H,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, R, S>,
        IndependentEventSampler<H, D, R, S>,
    > for IndependentActiveLineageSampler<H, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_reference.is_some() as usize
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            D,
            R,
            S,
            IndependentCoalescenceSampler<H, R, S>,
            IndependentEventSampler<H, D, R, S>,
        >,
        rng: &mut impl Rng,
    ) -> Option<(R, Location, f64)> {
        let chosen_lineage_reference = match self.active_lineage_reference.take() {
            Some(chosen_active_lineage) => chosen_active_lineage,
            None => return None,
        };

        let lineage_location = simulation
            .lineage_store
            .extract_lineage_from_its_location(chosen_lineage_reference.clone());

        // TODO: As we are only doing geometric sampling for now, need to immediately increment discrete time step
        // TODO: How do we choose the time step for now?
        // TODO: Need to prime incoherent RNG here with location, discrete time step and substep 0

        // TODO: Need to get time to next event in while loop with exponential (simplest option)

        let event_time = time + rng.sample_exponential(0.5_f64);

        // TODO: Need to prime incoherent RNG here with location, discrete time step and substep 0

        let unique_event_time: f64 = if event_time > time {
            event_time
        } else {
            time.next_after(f64::INFINITY)
        };

        simulation
            .lineage_store
            .update_lineage_time_of_last_event(chosen_lineage_reference.clone(), unique_event_time);

        Some((
            chosen_lineage_reference,
            lineage_location,
            unique_event_time,
        ))
    }

    #[debug_requires(
        self.number_active_lineages() == 0,
        "does not overwrite the independent lineage"
    )]
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
            IndependentCoalescenceSampler<H, R, S>,
            IndependentEventSampler<H, D, R, S>,
        >,
        rng: &mut impl Rng,
    ) {
        let index_at_location =
            IndependentCoalescenceSampler::<H, R, S>::sample_coalescence_index_at_location(
                &location,
                simulation.habitat,
                rng,
            );

        simulation
            .lineage_store
            .insert_lineage_to_location_at_index(
                lineage_reference.clone(),
                location,
                index_at_location,
            );

        self.active_lineage_reference = Some(lineage_reference);
    }
}

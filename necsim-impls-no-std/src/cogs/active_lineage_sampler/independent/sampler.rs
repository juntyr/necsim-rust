use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, DispersalSampler, HabitatToU64Injection, IncoherentLineageStore,
    LineageReference, PrimeableRng,
};
use necsim_core::intrinsics::{exp, floor};
use necsim_core::landscape::IndexedLocation;
use necsim_core::simulation::partial::active_lineager_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler;
use crate::cogs::event_sampler::independent::IndependentEventSampler;

use super::IndependentActiveLineageSampler;

#[contract_trait]
impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    ActiveLineageSampler<
        H,
        G,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, G, R, S>,
        IndependentEventSampler<H, G, D, R, S>,
    > for IndependentActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_reference.is_some() as usize
    }

    fn get_time_of_last_event(&self, lineage_store: &S) -> f64 {
        self.active_lineage_reference
            .as_ref()
            .and_then(|lineage_reference| lineage_store.get(lineage_reference.clone()))
            .map_or(0.0_f64, |lineage| lineage.time_of_last_event())
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
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, D, R, S>,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        use necsim_core::cogs::RngSampler;

        let chosen_lineage_reference = match self.active_lineage_reference.take() {
            Some(chosen_active_lineage) => chosen_active_lineage,
            None => return None,
        };

        #[allow(clippy::question_mark)]
        if simulation
            .lineage_store
            .get(chosen_lineage_reference.clone())
            .is_none()
        {
            // Check for extraneously simulated lineages
            return None;
        }

        let lineage_indexed_location = simulation
            .lineage_store
            .extract_lineage_from_its_location(chosen_lineage_reference.clone());

        let delta_t = 0.1_f64;
        let lambda = 0.5_f64;

        let p = 1.0_f64 - exp(-lambda * delta_t);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = floor(time / delta_t) as u64 + 1;

        loop {
            rng.prime_with(simulation.habitat, &lineage_indexed_location, time_step);

            if rng.sample_event(p) {
                break;
            }

            time_step += 1;
        }

        #[allow(clippy::cast_precision_loss)]
        let event_time = (time_step as f64) * delta_t;

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
            lineage_indexed_location,
            unique_event_time,
        ))
    }

    #[debug_requires(
        self.number_active_lineages() == 0,
        "does not overwrite the independent lineage"
    )]
    #[allow(clippy::type_complexity)]
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
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, D, R, S>,
        >,
        _rng: &mut G,
    ) {
        simulation
            .lineage_store
            .insert_lineage_to_indexed_location(lineage_reference.clone(), indexed_location);

        self.active_lineage_reference = Some(lineage_reference);
    }
}

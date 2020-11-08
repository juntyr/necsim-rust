use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, DispersalSampler, Habitat, IncoherentLineageStore, LineageReference,
};
use necsim_core::intrinsics::{exp, floor};
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

        #[allow(clippy::question_mark)]
        if simulation
            .lineage_store
            .get(chosen_lineage_reference.clone())
            .is_none()
        {
            // Check for extraneously simulated lineages
            return None;
        }

        let lineage_location = simulation
            .lineage_store
            .extract_lineage_from_its_location(chosen_lineage_reference.clone());

        let delta_t = 0.1_f64;
        let lambda = 0.5_f64;

        let p = 1.0_f64 - exp(-lambda * delta_t);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = floor(time / delta_t) as u64 + 1;

        loop {
            /*let location_x_bytes = lineage_location.x().to_le_bytes();
            let location_y_bytes = lineage_location.y().to_le_bytes();
            let time_step_bytes = time_step.to_le_bytes();

            rng.prime_with([
                location_x_bytes[0],
                location_x_bytes[1],
                location_x_bytes[2],
                location_x_bytes[3],
                location_y_bytes[0],
                location_y_bytes[1],
                location_y_bytes[2],
                location_y_bytes[3],
                time_step_bytes[0],
                time_step_bytes[1],
                time_step_bytes[2],
                time_step_bytes[3],
                time_step_bytes[4],
                time_step_bytes[5],
                time_step_bytes[6],
                time_step_bytes[7],
            ]);*/

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

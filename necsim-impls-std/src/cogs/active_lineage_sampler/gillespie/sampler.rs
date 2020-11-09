use float_next_after::NextAfter;

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, CoherentLineageStore, DispersalSampler, Habitat,
    LineageReference, RngCore,
};
use necsim_core::landscape::Location;
use necsim_core::simulation::partial::active_lineager_sampler::PartialSimulation;

use necsim_impls_no_std::cogs::event_sampler::gillespie::GillespieEventSampler;

use super::{EventTime, GillespieActiveLineageSampler};

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
        E: GillespieEventSampler<H, G, D, R, S, C>,
    > ActiveLineageSampler<H, G, D, R, S, C, E>
    for GillespieActiveLineageSampler<H, G, D, R, S, C, E>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn pop_active_lineage_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<H, G, D, R, S, C, E>,
        rng: &mut G,
    ) -> Option<(R, Location, f64)> {
        use necsim_core::cogs::RngSampler;

        let (chosen_active_location, chosen_event_time) = match self.active_locations.pop() {
            Some((chosen_active_location, chosen_event_time)) => {
                (chosen_active_location, chosen_event_time.into())
            }
            None => return None,
        };

        let lineages_at_location = simulation
            .lineage_store
            .get_active_lineages_at_location(&chosen_active_location);
        let number_lineages_left_at_location = lineages_at_location.len() - 1;

        let chosen_lineage_index_at_location = rng.sample_index(lineages_at_location.len());
        let chosen_lineage_reference =
            lineages_at_location[chosen_lineage_index_at_location].clone();

        let lineage_location = simulation
            .lineage_store
            .pop_lineage_from_its_location(chosen_lineage_reference.clone());
        self.number_active_lineages -= 1;

        let unique_event_time: f64 = if chosen_event_time > time {
            chosen_event_time
        } else {
            time.next_after(f64::INFINITY)
        };

        if number_lineages_left_at_location > 0 {
            let event_rate_at_location =
                simulation.with_split_event_sampler(|event_sampler, simulation| {
                    event_sampler.get_event_rate_at_location(
                        &chosen_active_location,
                        simulation,
                        true, // all lineages that are left are in the store
                    )
                });

            self.active_locations.push(
                chosen_active_location,
                EventTime::from(unique_event_time + rng.sample_exponential(event_rate_at_location)),
            );
        }

        simulation
            .lineage_store
            .update_lineage_time_of_last_event(chosen_lineage_reference.clone(), unique_event_time);

        Some((
            chosen_lineage_reference,
            lineage_location,
            unique_event_time,
        ))
    }

    fn push_active_lineage_to_location(
        &mut self,
        lineage_reference: R,
        location: Location,
        time: f64,
        simulation: &mut PartialSimulation<H, G, D, R, S, C, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        simulation
            .lineage_store
            .append_lineage_to_location(lineage_reference, location.clone());

        let event_rate_at_location =
            simulation.with_split_event_sampler(|event_sampler, simulation| {
                event_sampler.get_event_rate_at_location(
                    &location, simulation,
                    true, // all lineages including lineage_reference are (back) in the store
                )
            });

        self.active_locations.push(
            location,
            EventTime::from(time + rng.sample_exponential(event_rate_at_location)),
        );

        self.number_active_lineages += 1;
    }
}

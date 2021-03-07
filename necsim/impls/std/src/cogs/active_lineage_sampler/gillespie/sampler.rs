use float_next_after::NextAfter;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, CoherentLineageStore, DispersalSampler,
        EmigrationExit, EmptyActiveLineageSamplerError, Habitat, ImmigrationEntry,
        LineageReference, PeekableActiveLineageSampler, RngCore, SpeciationProbability,
    },
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

use necsim_impls_no_std::cogs::event_sampler::gillespie::GillespieEventSampler;

use super::{EventTime, GillespieActiveLineageSampler};

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, R, S>,
        E: GillespieEventSampler<H, G, N, D, R, S, X, C>,
        I: ImmigrationEntry,
    > ActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
    for GillespieActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn get_time_of_last_event(&self) -> f64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        use necsim_core::cogs::RngSampler;

        let (chosen_active_location, chosen_event_time) = match self.active_locations.pop() {
            Some((chosen_active_location, chosen_event_time)) => {
                (chosen_active_location, chosen_event_time.into())
            },
            None => return None,
        };

        let lineages_at_location = simulation
            .lineage_store
            .get_active_local_lineage_references_at_location_unordered(
                &chosen_active_location,
                &simulation.habitat,
            );
        let number_lineages_left_at_location = lineages_at_location.len() - 1;

        let chosen_lineage_index_at_location = rng.sample_index(lineages_at_location.len());
        let chosen_lineage_reference =
            lineages_at_location[chosen_lineage_index_at_location].clone();

        let lineage_indexed_location = simulation
            .lineage_store
            .extract_lineage_from_its_location_coherent(
                chosen_lineage_reference.clone(),
                &simulation.habitat,
            );
        self.number_active_lineages -= 1;

        let unique_event_time: f64 = if chosen_event_time > self.last_event_time {
            chosen_event_time
        } else {
            self.last_event_time.next_after(f64::INFINITY)
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

        self.last_event_time = unique_event_time;

        Some((
            chosen_lineage_reference,
            lineage_indexed_location,
            unique_event_time,
        ))
    }

    #[debug_requires(
        simulation.lineage_store.get_active_local_lineage_references_at_location_unordered(
            indexed_location.location(), &simulation.habitat
        ).len() < (
            simulation.habitat.get_habitat_at_location(indexed_location.location()) as usize
        ), "location has habitat capacity for the lineage"
    )]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        let location = indexed_location.location().clone();

        simulation
            .lineage_store
            .insert_lineage_to_indexed_location_coherent(
                lineage_reference,
                indexed_location,
                &simulation.habitat,
            );

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

        self.last_event_time = time;

        self.number_active_lineages += 1;
    }

    fn insert_new_lineage_to_indexed_location(
        &mut self,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S, X, C, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        let location = indexed_location.location().clone();

        let _immigrant_lineage_reference = simulation.lineage_store.immigrate(
            &simulation.habitat,
            global_reference,
            indexed_location,
            time,
        );

        let event_rate_at_location =
            simulation.with_split_event_sampler(|event_sampler, simulation| {
                event_sampler.get_event_rate_at_location(
                    &location, simulation,
                    true, /* all lineages including _immigrant_lineage_reference
                          *   are (back) in the store */
                )
            });

        self.active_locations.push(
            location,
            EventTime::from(time + rng.sample_exponential(event_rate_at_location)),
        );

        self.last_event_time = time;

        self.number_active_lineages += 1;
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, R, S>,
        E: GillespieEventSampler<H, G, N, D, R, S, X, C>,
        I: ImmigrationEntry,
    > PeekableActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
    for GillespieActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>
{
    fn peek_time_of_next_event(
        &mut self,
        _rng: &mut G,
    ) -> Result<f64, EmptyActiveLineageSamplerError> {
        self.active_locations
            .peek()
            .map(|(_, next_event_time)| next_event_time.clone().into())
            .ok_or(EmptyActiveLineageSamplerError)
    }
}

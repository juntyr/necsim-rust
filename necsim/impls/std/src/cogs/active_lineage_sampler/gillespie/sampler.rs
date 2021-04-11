use float_next_after::NextAfter;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit,
        EmptyActiveLineageSamplerError, GloballyCoherentLineageStore, Habitat, ImmigrationEntry,
        LineageReference, PeekableActiveLineageSampler, RngCore, SpeciationProbability,
        TurnoverRate,
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
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
    > ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
    for GillespieActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn get_last_event_time(&self) -> f64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        use necsim_core::cogs::RngSampler;

        let (chosen_active_location, chosen_event_time) = match self.active_locations.pop() {
            Some((chosen_active_location, chosen_event_time)) => {
                (chosen_active_location, chosen_event_time.into())
            },
            None => return None,
        };

        let unique_event_time: f64 = if chosen_event_time > self.last_event_time {
            chosen_event_time
        } else {
            self.last_event_time.next_after(f64::INFINITY)
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
            .extract_lineage_from_its_location_globally_coherent(
                chosen_lineage_reference.clone(),
                unique_event_time,
                &simulation.habitat,
            );
        self.number_active_lineages -= 1;

        if number_lineages_left_at_location > 0 {
            let event_rate_at_location =
                simulation.with_split_event_sampler(|event_sampler, simulation| {
                    // All active lineages which are left, which now excludes
                    //  chosen_lineage_reference, are still in the lineage store
                    event_sampler.get_event_rate_at_location(&chosen_active_location, simulation)
                });

            self.active_locations.push(
                chosen_active_location,
                EventTime::from(unique_event_time + rng.sample_exponential(event_rate_at_location)),
            );
        }

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
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        let location = indexed_location.location().clone();

        simulation
            .lineage_store
            .insert_lineage_to_indexed_location_globally_coherent(
                lineage_reference,
                indexed_location,
                &simulation.habitat,
            );

        let event_rate_at_location =
            simulation.with_split_event_sampler(|event_sampler, simulation| {
                // All active lineage references, including lineage_reference,
                //  are now (back) in the lineage store
                event_sampler.get_event_rate_at_location(&location, simulation)
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
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        let location = indexed_location.location().clone();

        let _immigrant_lineage_reference = simulation.lineage_store.immigrate_globally_coherent(
            &simulation.habitat,
            global_reference,
            indexed_location,
            time,
        );

        let event_rate_at_location =
            simulation.with_split_event_sampler(|event_sampler, simulation| {
                // All active lineages, including _immigrant_lineage_reference,
                //  are now in the lineage store
                event_sampler.get_event_rate_at_location(&location, simulation)
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
        R: LineageReference<H>,
        S: GloballyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: GillespieEventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
    > PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
    for GillespieActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
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

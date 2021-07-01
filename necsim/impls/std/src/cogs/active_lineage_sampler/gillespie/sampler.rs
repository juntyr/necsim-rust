use core::num::NonZeroUsize;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit,
        GloballyCoherentLineageStore, Habitat, ImmigrationEntry, LineageReference, MathsCore,
        RngCore, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_impls_no_std::cogs::event_sampler::gillespie::{
    GillespieEventSampler, GillespiePartialSimulation,
};

use super::{EventTime, GillespieActiveLineageSampler};

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
    for GillespieActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_and_event_time<F: FnOnce(PositiveF64) -> bool>(
        &mut self,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
        early_peek_stop: F,
    ) -> Option<(Lineage, PositiveF64)> {
        use necsim_core::cogs::RngSampler;

        let chosen_event_time = (*self.active_locations.peek()?.1).into();

        let next_event_time = PositiveF64::max_after(self.last_event_time, chosen_event_time);

        if early_peek_stop(next_event_time) {
            return None;
        }

        self.last_event_time = next_event_time.into();

        let chosen_active_location = self.active_locations.pop()?.0;

        let lineages_at_location = simulation
            .lineage_store
            .get_local_lineage_references_at_location_unordered(
                &chosen_active_location,
                &simulation.habitat,
            );
        let number_lineages_left_at_location = lineages_at_location.len() - 1;

        // Safety: `lineages_at_location` must be >0 since
        //         `chosen_active_location` can only be selected in that case
        let chosen_lineage_index_at_location =
            rng.sample_index(unsafe { NonZeroUsize::new_unchecked(lineages_at_location.len()) });
        let chosen_lineage_reference =
            lineages_at_location[chosen_lineage_index_at_location].clone();

        let mut chosen_lineage = simulation
            .lineage_store
            .extract_lineage_globally_coherent(chosen_lineage_reference, &simulation.habitat);
        chosen_lineage.last_event_time = self.last_event_time;

        self.number_active_lineages -= 1;

        if number_lineages_left_at_location > 0 {
            if let Ok(event_rate_at_location) = PositiveF64::new(
                simulation
                    .with_split_event_sampler(|event_sampler, simulation| {
                        GillespiePartialSimulation::without_emigration_exit(
                            simulation,
                            |simulation| {
                                // All active lineages which are left, which now excludes
                                //  chosen_lineage_reference, are still in the lineage store
                                event_sampler
                                    .get_event_rate_at_location(&chosen_active_location, simulation)
                            },
                        )
                    })
                    .get(),
            ) {
                self.active_locations.push(
                    chosen_active_location,
                    EventTime::from(
                        next_event_time + rng.sample_exponential(event_rate_at_location),
                    ),
                );
            }
        }

        Some((chosen_lineage, next_event_time))
    }

    #[debug_requires(
        simulation.lineage_store.get_local_lineage_references_at_location_unordered(
            lineage.indexed_location.location(), &simulation.habitat
        ).len() < (simulation.habitat.get_habitat_at_location(
            lineage.indexed_location.location()
        ) as usize), "location has habitat capacity for the lineage"
    )]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
    ) {
        use necsim_core::cogs::RngSampler;

        self.last_event_time = lineage.last_event_time;

        let location = lineage.indexed_location.location().clone();

        let _lineage_reference = simulation
            .lineage_store
            .insert_lineage_globally_coherent(lineage, &simulation.habitat);

        if let Ok(event_rate_at_location) = PositiveF64::new(
            simulation
                .with_split_event_sampler(|event_sampler, simulation| {
                    GillespiePartialSimulation::without_emigration_exit(simulation, |simulation| {
                        // All active lineage references, including lineage_reference,
                        //  are now (back) in the lineage store
                        event_sampler.get_event_rate_at_location(&location, simulation)
                    })
                })
                .get(),
        ) {
            self.active_locations.push(
                location,
                EventTime::from(
                    self.last_event_time + rng.sample_exponential(event_rate_at_location),
                ),
            );

            self.number_active_lineages += 1;
        }
    }
}

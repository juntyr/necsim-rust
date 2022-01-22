use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LocallyCoherentLineageStore, MathsCore,
        RngCore, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
    simulation::partial::active_lineage_sampler::PartialSimulation,
};

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use super::IndividualAliasActiveLineageSampler;

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
    for IndividualAliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    type LineageIterator<'a>
    where
        H: 'a,
        G: 'a,
        R: 'a,
        S: 'a,
        X: 'a,
        D: 'a,
        C: 'a,
        T: 'a,
        N: 'a,
        E: 'a,
        I: 'a,
    = impl Iterator<Item = &'a Lineage>;

    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    fn iter_active_lineages_ordered<'a>(
        &'a self,
        _habitat: &'a H,
        lineage_store: &'a S,
    ) -> Self::LineageIterator<'a> {
        self.alias_sampler
            .iter_all_events_ordered()
            .map(move |local_reference| &lineage_store[local_reference.clone()])
    }

    #[must_use]
    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_and_event_time<F: FnOnce(PositiveF64) -> ControlFlow<(), ()>>(
        &mut self,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        rng: &mut G,
        early_peek_stop: F,
    ) -> Option<(Lineage, PositiveF64)> {
        use necsim_core::cogs::RngSampler;

        let total_rate = self.alias_sampler.total_weight();

        if let Ok(lambda) = PositiveF64::new(total_rate.get()) {
            let event_time = self.last_event_time + rng.sample_exponential(lambda);

            let next_event_time = PositiveF64::max_after(self.last_event_time, event_time);

            if early_peek_stop(next_event_time).is_break() {
                return None;
            }

            self.last_event_time = next_event_time.into();

            // Note: This should always be Some
            let chosen_lineage_reference = self.alias_sampler.sample_pop(rng)?;

            let lineage = simulation
                .lineage_store
                .extract_lineage_locally_coherent(chosen_lineage_reference, &simulation.habitat);

            self.number_active_lineages -= 1;

            Some((lineage, next_event_time))
        } else {
            None
        }
    }

    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        _rng: &mut G,
    ) {
        self.last_event_time = lineage.last_event_time;

        let rate = simulation.turnover_rate.get_turnover_rate_at_location(
            lineage.indexed_location.location(),
            &simulation.habitat,
        );

        let lineage_reference = simulation
            .lineage_store
            .insert_lineage_locally_coherent(lineage, &simulation.habitat);

        if let Ok(event_rate) = PositiveF64::new(rate.get()) {
            self.alias_sampler.add_push(lineage_reference, event_rate);

            self.number_active_lineages += 1;
        }
    }
}

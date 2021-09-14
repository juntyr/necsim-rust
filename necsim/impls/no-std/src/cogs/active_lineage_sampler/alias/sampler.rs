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

use crate::cogs::event_sampler::gillespie::{GillespieEventSampler, GillespiePartialSimulation};

use super::AliasActiveLineageSampler;

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
    for AliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
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

        let total_rate = self.alias_sampler.total_weight();

        if let Ok(lambda) = PositiveF64::new(total_rate.get()) {
            let event_time = self.last_event_time + rng.sample_exponential(lambda);

            let next_event_time = PositiveF64::max_after(self.last_event_time, event_time);

            if early_peek_stop(next_event_time) {
                return None;
            }

            self.last_event_time = next_event_time.into();

            // Note: In practice, this should always return Some
            let chosen_lineage_reference = self.alias_sampler.sample_pop(rng)?;

            let chosen_lineage = simulation
                .lineage_store
                .extract_lineage_globally_coherent(chosen_lineage_reference, &simulation.habitat);
            self.number_active_lineages -= 1;

            Some((chosen_lineage, next_event_time))
        } else {
            None
        }
    }

    #[debug_requires(
        simulation.lineage_store.get_local_lineage_references_at_location_unordered(
            lineage.indexed_location.location(), &simulation.habitat
        ).len() < (
            simulation.habitat.get_habitat_at_location(
                lineage.indexed_location.location()
            ) as usize
        ), "location has habitat capacity for the lineage"
    )]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<M, H, G, R, S, X, D, C, T, N, E>,
        _rng: &mut G,
    ) {
        self.last_event_time = lineage.last_event_time;

        //let location = lineage.indexed_location.location().clone();

        let rate = simulation.turnover_rate.get_turnover_rate_at_location(lineage.indexed_location.location(), &simulation.habitat);

        let lineage_reference = simulation
            .lineage_store
            .insert_lineage_globally_coherent(lineage, &simulation.habitat);

        if let Ok(event_rate) = PositiveF64::new(rate.get()) {
            self.alias_sampler.add_push(lineage_reference, event_rate);

            self.number_active_lineages += 1;
        }
    }
}

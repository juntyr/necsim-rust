use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, PrimeableRng,
        SpeciationProbability, TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
    simulation::partial::active_lineager_sampler::PartialSimulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: PrimeableRng<F>,
        X: EmigrationExit<F, H, G, GlobalLineageReference, IndependentLineageStore<F, H>>,
        D: DispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        J: EventTimeSampler<F, H, G, T>,
    >
    ActiveLineageSampler<
        F,
        H,
        G,
        GlobalLineageReference,
        IndependentLineageStore<F, H>,
        X,
        D,
        IndependentCoalescenceSampler<F, H>,
        T,
        N,
        IndependentEventSampler<F, H, G, X, D, T, N>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<F, H, G, X, D, T, N, J>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage.is_some() as usize
    }

    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.active_lineage
            .as_ref()
            .map_or(NonNegativeF64::zero(), |lineage| lineage.last_event_time)
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn pop_active_lineage_and_event_time<W: FnOnce(PositiveF64) -> bool>(
        &mut self,
        simulation: &mut PartialSimulation<
            F,
            H,
            G,
            GlobalLineageReference,
            IndependentLineageStore<F, H>,
            X,
            D,
            IndependentCoalescenceSampler<F, H>,
            T,
            N,
            IndependentEventSampler<F, H, G, X, D, T, N>,
        >,
        rng: &mut G,
        early_peek_stop: W,
    ) -> Option<(Lineage, PositiveF64)> {
        if let Some(active_lineage) = &self.active_lineage {
            // Check for extraneously simulated (inactive) lineages
            let event_time = self
                .event_time_sampler
                .next_event_time_at_indexed_location_weakly_after(
                    &active_lineage.indexed_location,
                    active_lineage.last_event_time,
                    &simulation.habitat,
                    rng,
                    &simulation.turnover_rate,
                );

            let next_event_time =
                PositiveF64::max_after(active_lineage.last_event_time, event_time);

            if early_peek_stop(next_event_time) {
                return None;
            }

            // Note: Option::take would be better but uses local memory
            let chosen_lineage = active_lineage.clone();
            self.active_lineage = None;

            Some((chosen_lineage, next_event_time))
        } else {
            None
        }
    }

    #[debug_requires(
        self.number_active_lineages() == 0,
        "does not overwrite the independent lineage"
    )]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        _simulation: &mut PartialSimulation<
            F,
            H,
            G,
            GlobalLineageReference,
            IndependentLineageStore<F, H>,
            X,
            D,
            IndependentCoalescenceSampler<F, H>,
            T,
            N,
            IndependentEventSampler<F, H, G, X, D, T, N>,
        >,
        _rng: &mut G,
    ) {
        self.active_lineage = Some(lineage);
    }
}

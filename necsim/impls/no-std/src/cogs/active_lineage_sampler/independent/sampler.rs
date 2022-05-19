use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        distribution::UniformClosedOpenUnit, ActiveLineageSampler, DispersalSampler,
        EmigrationExit, Habitat, MathsCore, PrimeableRng, Rng, Samples, SpeciationProbability,
        TurnoverRate,
    },
    lineage::Lineage,
    simulation::partial::active_lineage_sampler::PartialSimulation,
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
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M, Generator: PrimeableRng> + Samples<M, UniformClosedOpenUnit>,
        X: EmigrationExit<M, H, G, IndependentLineageStore<M, H>>,
        D: DispersalSampler<M, H, G>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        J: EventTimeSampler<M, H, G, T>,
    >
    ActiveLineageSampler<
        M,
        H,
        G,
        IndependentLineageStore<M, H>,
        X,
        D,
        IndependentCoalescenceSampler<M, H>,
        T,
        N,
        IndependentEventSampler<M, H, G, X, D, T, N>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<M, H, G, X, D, T, N, J>
{
    type LineageIterator<'a> = impl Iterator<Item = &'a Lineage> where H: 'a, G: 'a, X: 'a, D: 'a, T: 'a, N: 'a, J: 'a;

    #[must_use]
    fn number_active_lineages(&self) -> usize {
        usize::from(self.active_lineage.is_some())
    }

    #[must_use]
    fn iter_active_lineages_ordered(
        &self,
        _habitat: &H,
        _lineage_store: &IndependentLineageStore<M, H>,
    ) -> Self::LineageIterator<'_> {
        self.active_lineage.iter()
    }

    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time.max(self.min_event_time)
    }

    #[must_use]
    #[inline]
    fn pop_active_lineage_and_event_time<F: FnOnce(PositiveF64) -> ControlFlow<(), ()>>(
        &mut self,
        simulation: &mut PartialSimulation<
            M,
            H,
            G,
            IndependentLineageStore<M, H>,
            X,
            D,
            IndependentCoalescenceSampler<M, H>,
            T,
            N,
            IndependentEventSampler<M, H, G, X, D, T, N>,
        >,
        rng: &mut G,
        early_peek_stop: F,
    ) -> Option<(Lineage, PositiveF64)> {
        if let Some(active_lineage) = &self.active_lineage {
            let before_next_event = active_lineage.last_event_time.max(self.min_event_time);

            // Check for extraneously simulated (inactive) lineages
            let event_time = self
                .event_time_sampler
                .next_event_time_at_indexed_location_weakly_after(
                    &active_lineage.indexed_location,
                    before_next_event,
                    &simulation.habitat,
                    rng,
                    &simulation.turnover_rate,
                );

            let next_event_time = PositiveF64::max_after(before_next_event, event_time);

            if early_peek_stop(next_event_time).is_break() {
                return None;
            }

            self.last_event_time = next_event_time.into();

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
    #[inline]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        _simulation: &mut PartialSimulation<
            M,
            H,
            G,
            IndependentLineageStore<M, H>,
            X,
            D,
            IndependentCoalescenceSampler<M, H>,
            T,
            N,
            IndependentEventSampler<M, H, G, X, D, T, N>,
        >,
        _rng: &mut G,
    ) {
        self.last_event_time = lineage.last_event_time;

        self.active_lineage = Some(lineage);
    }
}

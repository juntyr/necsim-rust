use core::{
    num::{NonZeroU64, NonZeroUsize},
    ops::ControlFlow,
};

use necsim_core::{
    cogs::{
        distribution::{Bernoulli, Exponential, IndexUsize, Lambda, Length, UniformClosedOpenUnit},
        ActiveLineageSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LocallyCoherentLineageStore, MathsCore, Rng, SampledDistribution, Samples,
        SpeciationProbability,
    },
    lineage::Lineage,
    simulation::partial::active_lineage_sampler::PartialSimulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::{
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::unconditional::UnconditionalEventSampler,
    turnover_rate::uniform::UniformTurnoverRate,
};

use super::ClassicalActiveLineageSampler;

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M>
            + Samples<M, Exponential>
            + Samples<M, IndexUsize>
            + Samples<M, Bernoulli>
            + Samples<M, UniformClosedOpenUnit>,
        S: LocallyCoherentLineageStore<M, H>,
        X: EmigrationExit<M, H, G, S>,
        D: DispersalSampler<M, H, G>,
        N: SpeciationProbability<M, H>,
        I: ImmigrationEntry<M>,
    >
    ActiveLineageSampler<
        M,
        H,
        G,
        S,
        X,
        D,
        UnconditionalCoalescenceSampler<M, H, S>,
        UniformTurnoverRate,
        N,
        UnconditionalEventSampler<
            M,
            H,
            G,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<M, H, S>,
            UniformTurnoverRate,
            N,
        >,
        I,
    > for ClassicalActiveLineageSampler<M, H, G, S, X, D, N, I>
{
    type LineageIterator<'a> = impl Iterator<Item = &'a Lineage> where H: 'a, G: 'a, S: 'a, X: 'a, D: 'a, N: 'a, I: 'a;

    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    #[must_use]
    fn iter_active_lineages_ordered<'a>(
        &'a self,
        _habitat: &'a H,
        lineage_store: &'a S,
    ) -> Self::LineageIterator<'a> {
        self.active_lineage_references
            .iter()
            .map(move |local_reference| &lineage_store[local_reference])
    }

    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time
    }

    #[must_use]
    fn pop_active_lineage_and_event_time<F: FnOnce(PositiveF64) -> ControlFlow<(), ()>>(
        &mut self,
        simulation: &mut PartialSimulation<
            M,
            H,
            G,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<M, H, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                M,
                H,
                G,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<M, H, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        rng: &mut G,
        early_peek_stop: F,
    ) -> Option<(Lineage, PositiveF64)> {
        if let Some(number_active_lineages) = NonZeroU64::new(self.number_active_lineages() as u64)
        {
            let lambda = simulation.turnover_rate.get_uniform_turnover_rate()
                * PositiveF64::from(number_active_lineages);

            let event_time = self.last_event_time + Exponential::sample_with(rng, Lambda(lambda));

            let next_event_time = PositiveF64::max_after(self.last_event_time, event_time);

            if early_peek_stop(next_event_time).is_break() {
                return None;
            }

            self.last_event_time = next_event_time.into();

            // Safety: The outer if statement has already shown that the number
            //         of remaining lineages is non-zero
            let chosen_lineage_index = IndexUsize::sample_with(
                rng,
                Length(unsafe {
                    NonZeroUsize::new_unchecked(self.active_lineage_references.len())
                }),
            );
            let chosen_lineage_reference = self
                .active_lineage_references
                .swap_remove(chosen_lineage_index);

            let chosen_lineage = simulation
                .lineage_store
                .extract_lineage_locally_coherent(chosen_lineage_reference, &simulation.habitat);

            Some((chosen_lineage, next_event_time))
        } else {
            None
        }
    }

    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<
            M,
            H,
            G,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<M, H, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                M,
                H,
                G,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<M, H, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        _rng: &mut G,
    ) {
        self.last_event_time = lineage.last_event_time;

        let lineage_reference = simulation
            .lineage_store
            .insert_lineage_locally_coherent(lineage, &simulation.habitat);

        self.active_lineage_references.push(lineage_reference);
    }
}

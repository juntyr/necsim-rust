use core::num::NonZeroU64;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, ImmigrationEntry,
        LineageReference, LocallyCoherentLineageStore, RngCore, SpeciationProbability,
    },
    lineage::Lineage,
    simulation::partial::active_lineager_sampler::PartialSimulation,
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
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        N: SpeciationProbability<F, H>,
        I: ImmigrationEntry<F>,
    >
    ActiveLineageSampler<
        F,
        H,
        G,
        R,
        S,
        X,
        D,
        UnconditionalCoalescenceSampler<F, H, R, S>,
        UniformTurnoverRate,
        N,
        UnconditionalEventSampler<
            F,
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<F, H, R, S>,
            UniformTurnoverRate,
            N,
        >,
        I,
    > for ClassicalActiveLineageSampler<F, H, G, R, S, X, D, N, I>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    fn get_last_event_time(&self) -> NonNegativeF64 {
        self.last_event_time
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_and_event_time<W: FnOnce(PositiveF64) -> bool>(
        &mut self,
        simulation: &mut PartialSimulation<
            F,
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<F, H, R, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                F,
                H,
                G,
                R,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<F, H, R, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        rng: &mut G,
        early_peek_stop: W,
    ) -> Option<(Lineage, PositiveF64)> {
        use necsim_core::cogs::RngSampler;

        if let Some(number_active_lineages) = NonZeroU64::new(self.number_active_lineages() as u64)
        {
            let lambda = UniformTurnoverRate::get_uniform_turnover_rate()
                * PositiveF64::from(number_active_lineages);

            let event_time = self.last_event_time + rng.sample_exponential(lambda);

            let next_event_time = PositiveF64::max_after(self.last_event_time, event_time);

            if early_peek_stop(next_event_time) {
                return None;
            }

            self.last_event_time = next_event_time.into();

            let chosen_lineage_index = rng.sample_index(self.active_lineage_references.len());
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

    #[allow(clippy::type_complexity, clippy::cast_possible_truncation)]
    fn push_active_lineage(
        &mut self,
        lineage: Lineage,
        simulation: &mut PartialSimulation<
            F,
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<F, H, R, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                F,
                H,
                G,
                R,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<F, H, R, S>,
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

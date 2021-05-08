use core::num::NonZeroU64;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, EmptyActiveLineageSamplerError,
        Habitat, ImmigrationEntry, LineageReference, LocallyCoherentLineageStore,
        PeekableActiveLineageSampler, RngCore, SpeciationProbability,
    },
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
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
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LocallyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        N: SpeciationProbability<H>,
        I: ImmigrationEntry,
    >
    ActiveLineageSampler<
        H,
        G,
        R,
        S,
        X,
        D,
        UnconditionalCoalescenceSampler<H, R, S>,
        UniformTurnoverRate,
        N,
        UnconditionalEventSampler<
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<H, R, S>,
            UniformTurnoverRate,
            N,
        >,
        I,
    > for ClassicalActiveLineageSampler<H, G, R, S, X, D, N, I>
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
    fn pop_active_lineage_indexed_location_prior_event_time(
        &mut self,
        simulation: &mut PartialSimulation<
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<H, R, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                H,
                G,
                R,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<H, R, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, NonNegativeF64, PositiveF64)> {
        use necsim_core::cogs::RngSampler;

        // The next event time must be calculated before the next active lineage is
        //  popped
        let optional_next_event_time =
            self.peek_time_of_next_event(&simulation.habitat, &simulation.turnover_rate, rng);

        let (next_event_time, last_active_lineage_reference) = match (
            optional_next_event_time,
            self.active_lineage_references.pop(),
        ) {
            (Ok(next_event_time), Some(reference)) => (next_event_time, reference),
            _ => return None, // In practice, this must match (None, None)
        };

        let chosen_active_lineage_index =
            rng.sample_index(self.active_lineage_references.len() + 1);

        let chosen_lineage_reference =
            if chosen_active_lineage_index == self.active_lineage_references.len() {
                last_active_lineage_reference
            } else {
                let chosen_lineage_reference =
                    self.active_lineage_references[chosen_active_lineage_index].clone();

                self.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                chosen_lineage_reference
            };

        let (lineage_indexed_location, prior_event_time) = simulation
            .lineage_store
            .extract_lineage_from_its_location_locally_coherent(
                chosen_lineage_reference.clone(),
                next_event_time,
                &simulation.habitat,
            );

        self.last_event_time = next_event_time.into();

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;

        Some((
            chosen_lineage_reference,
            lineage_indexed_location,
            prior_event_time,
            next_event_time,
        ))
    }

    #[allow(clippy::type_complexity, clippy::cast_possible_truncation)]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        time: PositiveF64,
        simulation: &mut PartialSimulation<
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<H, R, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                H,
                G,
                R,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<H, R, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        _rng: &mut G,
    ) {
        simulation
            .lineage_store
            .insert_lineage_to_indexed_location_locally_coherent(
                lineage_reference.clone(),
                indexed_location,
                &simulation.habitat,
            );

        self.active_lineage_references.push(lineage_reference);

        self.last_event_time = time.into();

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;
    }

    #[allow(clippy::type_complexity)]
    fn insert_new_lineage_to_indexed_location(
        &mut self,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time: PositiveF64,
        simulation: &mut PartialSimulation<
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<H, R, S>,
            UniformTurnoverRate,
            N,
            UnconditionalEventSampler<
                H,
                G,
                R,
                S,
                X,
                D,
                UnconditionalCoalescenceSampler<H, R, S>,
                UniformTurnoverRate,
                N,
            >,
        >,
        _rng: &mut G,
    ) {
        let immigrant_lineage_reference = simulation.lineage_store.immigrate_locally_coherent(
            &simulation.habitat,
            global_reference,
            indexed_location,
            time,
        );

        self.active_lineage_references
            .push(immigrant_lineage_reference);

        self.last_event_time = time.into();

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LocallyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        N: SpeciationProbability<H>,
        I: ImmigrationEntry,
    >
    PeekableActiveLineageSampler<
        H,
        G,
        R,
        S,
        X,
        D,
        UnconditionalCoalescenceSampler<H, R, S>,
        UniformTurnoverRate,
        N,
        UnconditionalEventSampler<
            H,
            G,
            R,
            S,
            X,
            D,
            UnconditionalCoalescenceSampler<H, R, S>,
            UniformTurnoverRate,
            N,
        >,
        I,
    > for ClassicalActiveLineageSampler<H, G, R, S, X, D, N, I>
{
    fn peek_time_of_next_event(
        &mut self,
        _habitat: &H,
        _turnover_rate: &UniformTurnoverRate,
        rng: &mut G,
    ) -> Result<PositiveF64, EmptyActiveLineageSamplerError> {
        use necsim_core::cogs::RngSampler;

        if self.next_event_time.is_none() {
            if let Some(number_active_lineages) =
                NonZeroU64::new(self.number_active_lineages() as u64)
            {
                let lambda = UniformTurnoverRate::get_uniform_turnover_rate()
                    * PositiveF64::from(number_active_lineages);

                let event_time = self.last_event_time + rng.sample_exponential(lambda);

                let unique_event_time = PositiveF64::max_after(self.last_event_time, event_time);

                self.next_event_time = Some(unique_event_time);
            }
        }

        self.next_event_time.ok_or(EmptyActiveLineageSamplerError)
    }
}

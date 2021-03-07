use float_next_after::NextAfter;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoherentLineageStore, DispersalSampler, EmigrationExit,
        EmptyActiveLineageSamplerError, Habitat, ImmigrationEntry, LineageReference,
        PeekableActiveLineageSampler, RngCore, SpeciationProbability,
    },
    landscape::IndexedLocation,
    lineage::GlobalLineageReference,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

use crate::cogs::{
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    event_sampler::unconditional::UnconditionalEventSampler,
};

use super::ClassicalActiveLineageSampler;

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        I: ImmigrationEntry,
    >
    ActiveLineageSampler<
        H,
        G,
        N,
        D,
        R,
        S,
        X,
        UnconditionalCoalescenceSampler<H, R, S>,
        UnconditionalEventSampler<H, G, N, D, R, S, X, UnconditionalCoalescenceSampler<H, R, S>>,
        I,
    > for ClassicalActiveLineageSampler<H, G, N, D, R, S, X, I>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    fn get_time_of_last_event(&self) -> f64 {
        self.last_event_time
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            X,
            UnconditionalCoalescenceSampler<H, R, S>,
            UnconditionalEventSampler<
                H,
                G,
                N,
                D,
                R,
                S,
                X,
                UnconditionalCoalescenceSampler<H, R, S>,
            >,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        use necsim_core::cogs::RngSampler;

        // The next event time must be calculated before the next active lineage is
        // popped
        let optional_next_event_time = self.peek_time_of_next_event(rng);

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

        let lineage_indexed_location = simulation
            .lineage_store
            .extract_lineage_from_its_location_coherent(
                chosen_lineage_reference.clone(),
                &simulation.habitat,
            );

        simulation
            .lineage_store
            .update_lineage_time_of_last_event(chosen_lineage_reference.clone(), next_event_time);

        self.last_event_time = next_event_time;

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;

        Some((
            chosen_lineage_reference,
            lineage_indexed_location,
            next_event_time,
        ))
    }

    #[allow(clippy::type_complexity, clippy::cast_possible_truncation)]
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
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            X,
            UnconditionalCoalescenceSampler<H, R, S>,
            UnconditionalEventSampler<
                H,
                G,
                N,
                D,
                R,
                S,
                X,
                UnconditionalCoalescenceSampler<H, R, S>,
            >,
        >,
        _rng: &mut G,
    ) {
        simulation
            .lineage_store
            .insert_lineage_to_indexed_location_coherent(
                lineage_reference.clone(),
                indexed_location,
                &simulation.habitat,
            );

        self.active_lineage_references.push(lineage_reference);

        self.last_event_time = time;

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;
    }

    #[allow(clippy::type_complexity)]
    fn insert_new_lineage_to_indexed_location(
        &mut self,
        global_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            X,
            UnconditionalCoalescenceSampler<H, R, S>,
            UnconditionalEventSampler<
                H,
                G,
                N,
                D,
                R,
                S,
                X,
                UnconditionalCoalescenceSampler<H, R, S>,
            >,
        >,
        _rng: &mut G,
    ) {
        let immigrant_lineage_reference = simulation.lineage_store.immigrate(
            &simulation.habitat,
            global_reference,
            indexed_location,
            time,
        );

        self.active_lineage_references
            .push(immigrant_lineage_reference);

        self.last_event_time = time;

        // Reset the next event time because the internal state has changed
        self.next_event_time = None;
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
        I: ImmigrationEntry,
    >
    PeekableActiveLineageSampler<
        H,
        G,
        N,
        D,
        R,
        S,
        X,
        UnconditionalCoalescenceSampler<H, R, S>,
        UnconditionalEventSampler<H, G, N, D, R, S, X, UnconditionalCoalescenceSampler<H, R, S>>,
        I,
    > for ClassicalActiveLineageSampler<H, G, N, D, R, S, X, I>
{
    fn peek_time_of_next_event(
        &mut self,
        rng: &mut G,
    ) -> Result<f64, EmptyActiveLineageSamplerError> {
        use necsim_core::cogs::RngSampler;

        if self.next_event_time.is_none() && !self.active_lineage_references.is_empty() {
            // Assumption: This method is called before the next active lineage is popped
            #[allow(clippy::cast_precision_loss)]
            let lambda = 0.5_f64 * (self.number_active_lineages() as f64);

            let event_time = self.last_event_time + rng.sample_exponential(lambda);

            let unique_event_time: f64 = if event_time > self.last_event_time {
                event_time
            } else {
                self.last_event_time.next_after(f64::INFINITY)
            };

            self.next_event_time = Some(unique_event_time);
        }

        self.next_event_time.ok_or(EmptyActiveLineageSamplerError)
    }
}

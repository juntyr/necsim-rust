use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, Habitat, PrimeableRng,
        SpeciationProbability,
    },
    landscape::IndexedLocation,
    lineage::{GlobalLineageReference, Lineage},
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

#[contract_trait]
impl<
        H: Habitat,
        G: PrimeableRng<H>,
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        X: EmigrationExit<H, G, N, D, GlobalLineageReference, IndependentLineageStore<H>>,
    >
    ActiveLineageSampler<
        H,
        G,
        N,
        D,
        GlobalLineageReference,
        IndependentLineageStore<H>,
        X,
        IndependentCoalescenceSampler<H>,
        IndependentEventSampler<H, G, N, D, X>,
        NeverImmigrationEntry,
    > for IndependentActiveLineageSampler<H, G, N, T, D, X>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage
            .as_ref()
            .map_or(0, |lineage| lineage.is_active() as usize)
    }

    fn get_time_of_last_event(&self) -> f64 {
        self.active_lineage
            .as_ref()
            .map_or(0.0_f64, Lineage::time_of_last_event)
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            GlobalLineageReference,
            IndependentLineageStore<H>,
            X,
            IndependentCoalescenceSampler<H>,
            IndependentEventSampler<H, G, N, D, X>,
        >,
        rng: &mut G,
    ) -> Option<(GlobalLineageReference, IndexedLocation, f64)> {
        let chosen_lineage = match self.active_lineage {
            Some(ref mut chosen_lineage) => chosen_lineage,
            None => return None,
        };

        // Check for extraneously simulated (inactive) lineages
        let lineage_indexed_location = match unsafe { chosen_lineage.try_remove_from_location() } {
            Some(lineage_indexed_location) => lineage_indexed_location,
            None => return None,
        };

        let next_event_time = self
            .event_time_sampler
            .next_event_time_at_indexed_location_after(
                &lineage_indexed_location,
                chosen_lineage.time_of_last_event(),
                &simulation.habitat,
                rng,
            );

        unsafe { chosen_lineage.update_time_of_last_event(next_event_time) };

        Some((
            chosen_lineage.global_reference().clone(),
            lineage_indexed_location,
            next_event_time,
        ))
    }

    #[debug_requires(
        self.number_active_lineages() == 0,
        "does not overwrite the independent lineage"
    )]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        _lineage_reference: GlobalLineageReference,
        indexed_location: IndexedLocation,
        _time: f64,
        _simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            GlobalLineageReference,
            IndependentLineageStore<H>,
            X,
            IndependentCoalescenceSampler<H>,
            IndependentEventSampler<H, G, N, D, X>,
        >,
        _rng: &mut G,
    ) {
        if let Some(active_lineage) = &mut self.active_lineage {
            unsafe { active_lineage.move_to_indexed_location(indexed_location) }
        }
    }

    #[allow(clippy::type_complexity)]
    fn insert_new_lineage_to_indexed_location(
        &mut self,
        _global_reference: GlobalLineageReference,
        _indexed_location: IndexedLocation,
        _time: f64,
        _simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            GlobalLineageReference,
            IndependentLineageStore<H>,
            X,
            IndependentCoalescenceSampler<H>,
            IndependentEventSampler<H, G, N, D, X>,
        >,
        _rng: &mut G,
    ) {
        // Ignoring this call is only valid because there will never be any
        //  dynamic immigration in the independent algorithm
    }
}

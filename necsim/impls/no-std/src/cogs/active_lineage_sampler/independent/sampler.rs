use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, HabitatToU64Injection, IncoherentLineageStore,
        LineageReference, PrimeableRng,
    },
    landscape::IndexedLocation,
    simulation::partial::active_lineager_sampler::PartialSimulation,
};

use crate::cogs::{
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    event_sampler::independent::IndependentEventSampler,
};

use super::{EventTimeSampler, IndependentActiveLineageSampler};

#[contract_trait]
impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    ActiveLineageSampler<
        H,
        G,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, G, R, S>,
        IndependentEventSampler<H, G, D, R, S>,
    > for IndependentActiveLineageSampler<H, G, T, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.active_lineage_reference.is_some() as usize
    }

    fn get_time_of_last_event(&self, lineage_store: &S) -> f64 {
        self.active_lineage_reference
            .as_ref()
            .and_then(|lineage_reference| lineage_store.get(lineage_reference.clone()))
            .map_or(0.0_f64, necsim_core::lineage::Lineage::time_of_last_event)
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            D,
            R,
            S,
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, D, R, S>,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        let chosen_lineage_reference = match self.active_lineage_reference.take() {
            Some(chosen_active_lineage) => chosen_active_lineage,
            None => return None,
        };

        // Check for extraneously simulated lineages
        match simulation
            .lineage_store
            .get(chosen_lineage_reference.clone())
        {
            Some(lineage) if lineage.is_active() => (),
            _ => return None,
        }

        let lineage_indexed_location = simulation
            .lineage_store
            .extract_lineage_from_its_location(chosen_lineage_reference.clone());

        let next_event_time = self
            .event_time_sampler
            .next_event_time_at_indexed_location_after(
                &lineage_indexed_location,
                time,
                &simulation.habitat,
                rng,
            );

        simulation
            .lineage_store
            .update_lineage_time_of_last_event(chosen_lineage_reference.clone(), next_event_time);

        Some((
            chosen_lineage_reference,
            lineage_indexed_location,
            next_event_time,
        ))
    }

    #[debug_requires(
        self.number_active_lineages() == 0,
        "does not overwrite the independent lineage"
    )]
    #[allow(clippy::type_complexity)]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        _time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            D,
            R,
            S,
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, D, R, S>,
        >,
        _rng: &mut G,
    ) {
        simulation
            .lineage_store
            .insert_lineage_to_indexed_location(lineage_reference.clone(), indexed_location);

        self.active_lineage_reference = Some(lineage_reference);
    }
}

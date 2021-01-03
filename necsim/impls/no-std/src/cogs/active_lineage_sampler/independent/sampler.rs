use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, HabitatToU64Injection, IncoherentLineageStore,
        LineageReference, PrimeableRng, SpeciationProbability,
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
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    >
    ActiveLineageSampler<
        H,
        G,
        N,
        D,
        R,
        S,
        IndependentCoalescenceSampler<H, G, R, S>,
        IndependentEventSampler<H, G, N, D, R, S>,
    > for IndependentActiveLineageSampler<H, G, N, T, D, R, S>
{
    #[must_use]
    fn number_active_lineages(&self) -> usize {
        self.lineage_indexed_location.is_some() as usize
    }

    fn get_time_of_last_event(&self) -> f64 {
        self.lineage_time_of_last_event
    }

    #[must_use]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, N, D, R, S>,
        >,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)> {
        let chosen_lineage_reference = match self.active_lineage_reference {
            Some(ref chosen_active_lineage) => chosen_active_lineage.clone(),
            None => return None,
        };

        // Check for extraneously simulated (inactive) lineages
        let lineage_indexed_location = match self.lineage_indexed_location.take() {
            Some(lineage_indexed_location) => lineage_indexed_location,
            None => return None,
        };

        let next_event_time = self
            .event_time_sampler
            .next_event_time_at_indexed_location_after(
                &lineage_indexed_location,
                time,
                &simulation.habitat,
                rng,
            );

        self.lineage_time_of_last_event = next_event_time;

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
    #[debug_requires(
        self.active_lineage_reference == Some(lineage_reference.clone()),
        "does not introduce a new lineage reference"
    )]
    #[allow(clippy::type_complexity)]
    #[inline]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        _time: f64,
        _simulation: &mut PartialSimulation<
            H,
            G,
            N,
            D,
            R,
            S,
            IndependentCoalescenceSampler<H, G, R, S>,
            IndependentEventSampler<H, G, N, D, R, S>,
        >,
        _rng: &mut G,
    ) {
        self.lineage_indexed_location = Some(indexed_location);

        self.active_lineage_reference = Some(lineage_reference);
    }
}

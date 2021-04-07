use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoalescenceSampler, CoherentLineageStore, DispersalSampler,
        EmigrationExit, EventSampler, Habitat, LineageReference, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    event::{DispersalEvent, PackedEvent, SpeciationEvent},
    landscape::{IndexedLocation, Location},
    simulation::partial::event_sampler::PartialSimulation,
};

use super::GillespieEventSampler;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct UnconditionalGillespieEventSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
>(PhantomData<(H, G, R, S, X, D, C, T, N)>);

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Default for UnconditionalGillespieEventSampler<H, G, R, S, X, D, C, T, N>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, R, S, X, D, C, T, N)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Backup for UnconditionalGillespieEventSampler<H, G, R, S, X, D, C, T, N>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(H, G, R, S, X, D, C, T, N)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > EventSampler<H, G, R, S, X, D, C, T, N>
    for UnconditionalGillespieEventSampler<H, G, R, S, X, D, C, T, N>
{
    #[must_use]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        Some(event.global_lineage_reference().clone()) == old(
            simulation.lineage_store.get(lineage_reference.clone()).map(
                |lineage| lineage.global_reference().clone()
            )
        )
    }), "event occurs for lineage_reference")]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &mut PartialSimulation<H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
    ) -> Option<PackedEvent> {
        use necsim_core::cogs::RngSampler;

        let dispersal_origin = indexed_location;

        if rng.sample_event(
            simulation
                .speciation_probability
                .get_speciation_probability_at_location(
                    dispersal_origin.location(),
                    &simulation.habitat,
                ),
        ) {
            Some(
                SpeciationEvent {
                    origin: dispersal_origin,
                    time: event_time,
                    global_lineage_reference: simulation.lineage_store[lineage_reference]
                        .global_reference()
                        .clone(),
                }
                .into(),
            )
        } else {
            let dispersal_target = simulation.dispersal_sampler.sample_dispersal_from_location(
                dispersal_origin.location(),
                &simulation.habitat,
                rng,
            );

            // Check for emigration and return None iff lineage emigrated
            let (lineage_reference, dispersal_origin, dispersal_target, event_time) = simulation
                .with_mut_split_emigration_exit(|emigration_exit, simulation| {
                    emigration_exit.optionally_emigrate(
                        lineage_reference,
                        dispersal_origin,
                        dispersal_target,
                        event_time,
                        simulation,
                        rng,
                    )
                })?;

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    CoalescenceRngSample::new(rng),
                );

            Some(
                DispersalEvent {
                    origin: dispersal_origin,
                    time: event_time,
                    global_lineage_reference: simulation.lineage_store[lineage_reference]
                        .global_reference()
                        .clone(),
                    target: dispersal_target,
                    coalescence: optional_coalescence,
                }
                .into(),
            )
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > GillespieEventSampler<H, G, R, S, X, D, C, T, N>
    for UnconditionalGillespieEventSampler<H, G, R, S, X, D, C, T, N>
{
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, R, S, X, D, C, T, N>,
        lineage_store_includes_self: bool,
    ) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let population = (simulation
            .lineage_store
            .get_active_local_lineage_references_at_location_unordered(
                location,
                &simulation.habitat,
            )
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        population
            * simulation
                .turnover_rate
                .get_turnover_rate_at_location(location, &simulation.habitat)
    }
}

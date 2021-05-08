use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoalescenceSampler, DispersalSampler, EmigrationExit,
        EventSampler, Habitat, LineageReference, LocallyCoherentLineageStore, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, PackedEvent, SpeciationEvent},
    landscape::IndexedLocation,
    simulation::partial::event_sampler::PartialSimulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct UnconditionalEventSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: LocallyCoherentLineageStore<H, R>,
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
        S: LocallyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Default for UnconditionalEventSampler<H, G, R, S, X, D, C, T, N>
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
        S: LocallyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Backup for UnconditionalEventSampler<H, G, R, S, X, D, C, T, N>
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
        S: LocallyCoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > EventSampler<H, G, R, S, X, D, C, T, N>
    for UnconditionalEventSampler<H, G, R, S, X, D, C, T, N>
{
    #[must_use]
    #[debug_ensures(ret.as_ref().map_or(true, |event: &PackedEvent| {
        Some(event.global_lineage_reference.clone()) == old(
            simulation.lineage_store.get(lineage_reference.clone()).map(
                |lineage| lineage.global_reference().clone()
            )
        )
    }), "event occurs for lineage_reference")]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        prior_time: NonNegativeF64,
        event_time: PositiveF64,
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
                    prior_time,
                    event_time,
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
            let (lineage_reference, dispersal_origin, dispersal_target, prior_time, event_time) =
                simulation.with_mut_split_emigration_exit(|emigration_exit, simulation| {
                    emigration_exit.optionally_emigrate(
                        lineage_reference,
                        dispersal_origin,
                        dispersal_target,
                        prior_time,
                        event_time,
                        simulation,
                        rng,
                    )
                })?;

            let (dispersal_target, interaction) = simulation
                .coalescence_sampler
                .sample_interaction_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    CoalescenceRngSample::new(rng),
                );

            Some(
                DispersalEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: simulation.lineage_store[lineage_reference]
                        .global_reference()
                        .clone(),
                    target: dispersal_target,
                    interaction,
                }
                .into(),
            )
        }
    }
}

use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, event_sampler::EventHandler, Backup,
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, F64Core, Habitat,
        LineageReference, LocallyCoherentLineageStore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    lineage::Lineage,
    simulation::partial::event_sampler::PartialSimulation,
};
use necsim_core_bond::PositiveF64;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct UnconditionalEventSampler<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: LocallyCoherentLineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: DispersalSampler<F, H, G>,
    C: CoalescenceSampler<F, H, R, S>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
>(PhantomData<(F, H, G, R, S, X, D, C, T, N)>);

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > Default for UnconditionalEventSampler<F, H, G, R, S, X, D, C, T, N>
{
    fn default() -> Self {
        Self(PhantomData::<(F, H, G, R, S, X, D, C, T, N)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > Backup for UnconditionalEventSampler<F, H, G, R, S, X, D, C, T, N>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(F, H, G, R, S, X, D, C, T, N)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > EventSampler<F, H, G, R, S, X, D, C, T, N>
    for UnconditionalEventSampler<F, H, G, R, S, X, D, C, T, N>
{
    #[must_use]
    fn sample_event_for_lineage_at_event_time_or_emigrate<
        Q,
        Aux,
        FS: FnOnce(SpeciationEvent, Aux) -> Q,
        FD: FnOnce(DispersalEvent, Aux) -> Q,
        FE: FnOnce(Aux) -> Q,
    >(
        &mut self,
        Lineage {
            global_reference,
            last_event_time: prior_time,
            indexed_location: dispersal_origin,
        }: Lineage,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<F, H, G, R, S, X, D, C, T, N>,
        rng: &mut G,
        EventHandler {
            speciation,
            dispersal,
            emigration,
        }: EventHandler<FS, FD, FE>,
        auxiliary: Aux,
    ) -> Q {
        use necsim_core::cogs::RngSampler;

        if rng.sample_event(
            simulation
                .speciation_probability
                .get_speciation_probability_at_location(
                    dispersal_origin.location(),
                    &simulation.habitat,
                ),
        ) {
            speciation(
                SpeciationEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: global_reference,
                },
                auxiliary,
            )
        } else {
            let dispersal_target = simulation.dispersal_sampler.sample_dispersal_from_location(
                dispersal_origin.location(),
                &simulation.habitat,
                rng,
            );

            // Check for emigration and return None iff lineage emigrated
            if let Some((
                global_reference,
                dispersal_origin,
                dispersal_target,
                prior_time,
                event_time,
            )) = simulation.with_mut_split_emigration_exit(|emigration_exit, simulation| {
                emigration_exit.optionally_emigrate(
                    global_reference,
                    dispersal_origin,
                    dispersal_target,
                    prior_time,
                    event_time,
                    simulation,
                    rng,
                )
            }) {
                let (dispersal_target, interaction) = simulation
                    .coalescence_sampler
                    .sample_interaction_at_location(
                        dispersal_target,
                        &simulation.habitat,
                        &simulation.lineage_store,
                        CoalescenceRngSample::new(rng),
                    );

                dispersal(
                    DispersalEvent {
                        origin: dispersal_origin,
                        prior_time,
                        event_time,
                        global_lineage_reference: global_reference,
                        target: dispersal_target,
                        interaction,
                    },
                    auxiliary,
                )
            } else {
                emigration(auxiliary)
            }
        }
    }
}

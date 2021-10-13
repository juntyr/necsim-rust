use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, event_sampler::EventHandler, Backup,
        CoalescenceSampler, EmigrationExit, EventSampler, F64Core, GloballyCoherentLineageStore,
        Habitat, LineageReference, RngCore, RngSampler, SeparableDispersalSampler,
        SpeciationProbability, TurnoverRate,
    },
    event::{DispersalEvent, SpeciationEvent},
    landscape::Location,
    lineage::{Lineage, LineageInteraction},
    simulation::partial::event_sampler::PartialSimulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::{GillespieEventSampler, GillespiePartialSimulation},
};

mod probability;

use probability::ProbabilityAtLocation;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct ConditionalGillespieEventSampler<
    F: F64Core,
    H: Habitat<F>,
    G: RngCore<F>,
    R: LineageReference<F, H>,
    S: GloballyCoherentLineageStore<F, H, R>,
    X: EmigrationExit<F, H, G, R, S>,
    D: SeparableDispersalSampler<F, H, G>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
>(PhantomData<(F, H, G, R, S, X, D, T, N)>);

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: SeparableDispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > Default for ConditionalGillespieEventSampler<F, H, G, R, S, X, D, T, N>
{
    fn default() -> Self {
        Self(PhantomData::<(F, H, G, R, S, X, D, T, N)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: SeparableDispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > Backup for ConditionalGillespieEventSampler<F, H, G, R, S, X, D, T, N>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(F, H, G, R, S, X, D, T, N)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: SeparableDispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > EventSampler<F, H, G, R, S, X, D, ConditionalCoalescenceSampler<F, H, R, S>, T, N>
    for ConditionalGillespieEventSampler<F, H, G, R, S, X, D, T, N>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
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
        simulation: &mut PartialSimulation<
            F,
            H,
            G,
            R,
            S,
            X,
            D,
            ConditionalCoalescenceSampler<F, H, R, S>,
            T,
            N,
        >,
        rng: &mut G,
        EventHandler {
            speciation,
            dispersal,
            emigration,
        }: EventHandler<FS, FD, FE>,
        auxiliary: Aux,
    ) -> Q {
        // The event is sampled after the active lineage has been removed from
        //  the lineage store, but it must be included in the calculation
        let probability_at_location =
            GillespiePartialSimulation::without_emigration_exit(simulation, |simulation| {
                ProbabilityAtLocation::new(dispersal_origin.location(), simulation, false)
            });

        let event_sample = probability_at_location.total() * rng.sample_uniform();

        if event_sample < probability_at_location.speciation() {
            // Speciation Event
            speciation(
                SpeciationEvent {
                    origin: dispersal_origin,
                    prior_time,
                    event_time,
                    global_lineage_reference: global_reference,
                },
                auxiliary,
            )
        } else if event_sample
            < (probability_at_location.speciation().get()
                + probability_at_location.out_dispersal().get())
        {
            // Out-Dispersal Event
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_non_self_dispersal_from_location(
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
        } else {
            // In-Coalescence Event
            let (dispersal_target, coalescence) =
                ConditionalCoalescenceSampler::sample_coalescence_at_location(
                    dispersal_origin.location().clone(),
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
                    interaction: LineageInteraction::Coalescence(coalescence),
                },
                auxiliary,
            )
        }
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: GloballyCoherentLineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: SeparableDispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
    > GillespieEventSampler<F, H, G, R, S, X, D, ConditionalCoalescenceSampler<F, H, R, S>, T, N>
    for ConditionalGillespieEventSampler<F, H, G, R, S, X, D, T, N>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &GillespiePartialSimulation<
            F,
            H,
            G,
            R,
            S,
            D,
            ConditionalCoalescenceSampler<F, H, R, S>,
            T,
            N,
        >,
    ) -> NonNegativeF64 {
        // By PRE, all active lineages, including self, are in the lineage store
        let probability_at_location = ProbabilityAtLocation::new(location, simulation, true);

        let population = NonNegativeF64::from(
            simulation
                .lineage_store
                .get_local_lineage_references_at_location_unordered(location, &simulation.habitat)
                .len(),
        );

        NonNegativeF64::from(probability_at_location.total())
            * population
            * simulation
                .turnover_rate
                .get_turnover_rate_at_location(location, &simulation.habitat)
    }
}

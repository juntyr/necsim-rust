use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, CoalescenceRngSample, CoalescenceSampler, CoherentLineageStore, EmigrationExit,
        EventSampler, Habitat, LineageReference, RngCore, RngSampler, SeparableDispersalSampler,
        SpeciationProbability, TurnoverRate,
    },
    event::{Event, EventType},
    landscape::{IndexedLocation, Location},
    simulation::partial::event_sampler::PartialSimulation,
};

use crate::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    event_sampler::gillespie::GillespieEventSampler,
};

mod probability;

use probability::ProbabilityAtLocation;

#[allow(clippy::module_name_repetitions, clippy::type_complexity)]
#[derive(Debug)]
pub struct ConditionalGillespieEventSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: SeparableDispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
>(PhantomData<(H, G, R, S, X, D, T, N)>);

impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: SeparableDispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Default for ConditionalGillespieEventSampler<H, G, R, S, X, D, T, N>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, R, S, X, D, T, N)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: SeparableDispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > Backup for ConditionalGillespieEventSampler<H, G, R, S, X, D, T, N>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(H, G, R, S, X, D, T, N)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: SeparableDispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > EventSampler<H, G, R, S, X, D, ConditionalCoalescenceSampler<H, R, S>, T, N>
    for ConditionalGillespieEventSampler<H, G, R, S, X, D, T, N>
{
    #[must_use]
    #[allow(clippy::double_parens)]
    #[allow(clippy::type_complexity)]
    #[debug_ensures(ret.as_ref().map_or(true, |event| {
        Some(event.global_lineage_reference().clone()) == old(
            simulation.lineage_store.get(lineage_reference.clone()).map(
                |lineage| lineage.global_reference().clone()
            )
        )
    }), "event occurs for lineage_reference")]
    #[debug_ensures(match &ret {
        Some(event) => match event.r#type() {
            EventType::Speciation => true,
            EventType::Dispersal {
                target,
                coalescence,
                ..
            } => ((event.origin() == target) -> coalescence.is_some()),
        },
        None => true,
    }, "always coalesces on self-dispersal")]
    fn sample_event_for_lineage_at_indexed_location_time_or_emigrate(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &mut PartialSimulation<
            H,
            G,
            R,
            S,
            X,
            D,
            ConditionalCoalescenceSampler<H, R, S>,
            T,
            N,
        >,
        rng: &mut G,
    ) -> Option<Event> {
        let dispersal_origin = indexed_location;

        let probability_at_location = ProbabilityAtLocation::new(
            dispersal_origin.location(),
            simulation,
            false, // lineage_reference was popped from the store
        );

        let event_sample = probability_at_location.total() * rng.sample_uniform();

        let (event_type, lineage_reference, dispersal_origin, event_time) =
            if event_sample < probability_at_location.speciation() {
                // Speciation Event
                (
                    EventType::Speciation,
                    lineage_reference,
                    dispersal_origin,
                    event_time,
                )
            } else if event_sample
                < (probability_at_location.speciation() + probability_at_location.out_dispersal())
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
                let (lineage_reference, dispersal_origin, dispersal_target, event_time) =
                    simulation.with_mut_split_emigration_exit(|emigration_exit, simulation| {
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

                (
                    EventType::Dispersal {
                        coalescence: optional_coalescence,
                        target: dispersal_target,
                    },
                    lineage_reference,
                    dispersal_origin,
                    event_time,
                )
            } else {
                // In-Coalescence Event
                let (dispersal_target, coalescence) =
                    ConditionalCoalescenceSampler::sample_coalescence_at_location(
                        dispersal_origin.location().clone(),
                        &simulation.habitat,
                        &simulation.lineage_store,
                        CoalescenceRngSample::new(rng),
                    );

                (
                    EventType::Dispersal {
                        coalescence: Some(coalescence),
                        target: dispersal_target,
                    },
                    lineage_reference,
                    dispersal_origin,
                    event_time,
                )
            };

        Some(Event::new(
            dispersal_origin,
            event_time,
            simulation.lineage_store[lineage_reference]
                .global_reference()
                .clone(),
            event_type,
        ))
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: SeparableDispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
    > GillespieEventSampler<H, G, R, S, X, D, ConditionalCoalescenceSampler<H, R, S>, T, N>
    for ConditionalGillespieEventSampler<H, G, R, S, X, D, T, N>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<
            H,
            G,
            R,
            S,
            X,
            D,
            ConditionalCoalescenceSampler<H, R, S>,
            T,
            N,
        >,
        lineage_store_includes_self: bool,
    ) -> f64 {
        let probability_at_location =
            ProbabilityAtLocation::new(location, simulation, lineage_store_includes_self);

        #[allow(clippy::cast_precision_loss)]
        let population = (simulation
            .lineage_store
            .get_active_local_lineage_references_at_location_unordered(
                location,
                &simulation.habitat,
            )
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        probability_at_location.total()
            * population
            * simulation
                .turnover_rate
                .get_turnover_rate_at_location(location, &simulation.habitat)
    }
}

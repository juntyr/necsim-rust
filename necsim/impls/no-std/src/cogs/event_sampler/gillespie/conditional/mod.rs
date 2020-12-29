use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceSampler, CoherentLineageStore, EventSampler, Habitat, LineageReference, RngCore,
        RngSampler, SeparableDispersalSampler, SpeciationProbability,
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

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct ConditionalGillespieEventSampler<
    H: Habitat,
    G: RngCore,
    D: SeparableDispersalSampler<H, G>,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(PhantomData<(H, G, D, R, S)>);

impl<
        H: Habitat,
        G: RngCore,
        D: SeparableDispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > Default for ConditionalGillespieEventSampler<H, G, D, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, D, R, S)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: SeparableDispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > EventSampler<H, G, N, D, R, S, ConditionalCoalescenceSampler<H, G, R, S>>
    for ConditionalGillespieEventSampler<H, G, D, R, S>
{
    #[must_use]
    #[allow(clippy::double_parens)]
    #[allow(clippy::type_complexity)]
    #[debug_ensures(match &ret.r#type() {
        EventType::Speciation => true,
        EventType::Dispersal {
            target,
            coalescence,
            ..
        } => ((ret.origin() == target) -> coalescence.is_some()),
    }, "always coalesces on self-dispersal")]
    fn sample_event_for_lineage_at_indexed_location_time(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, N, D, R, S, ConditionalCoalescenceSampler<H, G, R, S>>,
        rng: &mut G,
    ) -> Event<H, R> {
        let dispersal_origin = indexed_location;

        let probability_at_location = ProbabilityAtLocation::new(
            dispersal_origin.location(),
            simulation,
            false, // lineage_reference was popped from the store
        );

        let event_sample = probability_at_location.total() * rng.sample_uniform();

        let event_type = if event_sample < probability_at_location.speciation() {
            EventType::Speciation
        } else if event_sample
            < (probability_at_location.speciation() + probability_at_location.out_dispersal())
        {
            let dispersal_target = simulation
                .dispersal_sampler
                .sample_non_self_dispersal_from_location(dispersal_origin.location(), rng);

            let (dispersal_target, optional_coalescence) = simulation
                .coalescence_sampler
                .sample_optional_coalescence_at_location(
                    dispersal_target,
                    &simulation.habitat,
                    &simulation.lineage_store,
                    rng,
                );

            EventType::Dispersal {
                coalescence: optional_coalescence,
                target: dispersal_target,
                marker: PhantomData::<H>,
            }
        } else {
            let (dispersal_target, coalescence) =
                ConditionalCoalescenceSampler::sample_coalescence_at_location(
                    dispersal_origin.location().clone(),
                    &simulation.lineage_store,
                    rng,
                );

            EventType::Dispersal {
                coalescence: Some(coalescence),
                target: dispersal_target,
                marker: PhantomData::<H>,
            }
        };

        Event::new(dispersal_origin, event_time, lineage_reference, event_type)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: SeparableDispersalSampler<H, G>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
    > GillespieEventSampler<H, G, N, D, R, S, ConditionalCoalescenceSampler<H, G, R, S>>
    for ConditionalGillespieEventSampler<H, G, D, R, S>
{
    #[must_use]
    #[allow(clippy::type_complexity)]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, N, D, R, S, ConditionalCoalescenceSampler<H, G, R, S>>,
        lineage_store_includes_self: bool,
    ) -> f64 {
        let probability_at_location =
            ProbabilityAtLocation::new(location, simulation, lineage_store_includes_self);

        #[allow(clippy::cast_precision_loss)]
        let population = (simulation
            .lineage_store
            .get_active_lineages_at_location(location)
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        probability_at_location.total() * population * 0.5_f64
    }
}

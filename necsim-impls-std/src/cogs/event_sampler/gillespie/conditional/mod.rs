use necsim_core::cogs::{
    CoalescenceSampler, EventSampler, Habitat, LineageReference, LineageStore,
};
use necsim_core::event::{Event, EventType};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;
use necsim_core::simulation::partial::event_sampler::PartialSimulation;

use crate::cogs::coalescence_sampler::conditional::ConditionalCoalescenceSampler;
use crate::cogs::dispersal_sampler::separable::SeparableDispersalSampler;

use super::GillespieEventSampler;

mod probability;

use probability::ProbabilityAtLocation;

#[allow(clippy::module_name_repetitions)]
pub struct ConditionalGillespieEventSampler<
    H: Habitat,
    D: SeparableDispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
>(std::marker::PhantomData<(H, D, R, S)>);

impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    > Default for ConditionalGillespieEventSampler<H, D, R, S>
{
    fn default() -> Self {
        Self(std::marker::PhantomData::<(H, D, R, S)>)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    > EventSampler<H, D, R, S, ConditionalCoalescenceSampler<H, R, S>>
    for ConditionalGillespieEventSampler<H, D, R, S>
{
    #[must_use]
    #[allow(clippy::double_parens)]
    #[debug_ensures(match &ret.r#type() {
        EventType::Speciation => true,
        EventType::Dispersal {
            origin,
            target,
            coalescence,
            ..
        } => ((origin == target) -> coalescence.is_some()),
    }, "always coalesces on self-dispersal")]
    fn sample_event_for_lineage_at_time(
        &self,
        lineage_reference: R,
        event_time: f64,
        simulation: &PartialSimulation<H, D, R, S, ConditionalCoalescenceSampler<H, R, S>>,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let dispersal_origin = simulation.lineage_store[lineage_reference.clone()].location();

        let probability_at_location = ProbabilityAtLocation::new(
            dispersal_origin,
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
                .sample_non_self_dispersal_from_location(dispersal_origin, rng);

            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                coalescence: simulation
                    .coalescence_sampler
                    .sample_optional_coalescence_at_location(
                        &dispersal_target,
                        simulation.habitat,
                        simulation.lineage_store,
                        rng,
                    ),
                target: dispersal_target,
                _marker: std::marker::PhantomData::<H>,
            }
        } else {
            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                target: dispersal_origin.clone(),
                coalescence: Some(
                    ConditionalCoalescenceSampler::sample_coalescence_at_location(
                        dispersal_origin,
                        simulation.lineage_store,
                        rng,
                    ),
                ),
                _marker: std::marker::PhantomData::<H>,
            }
        };

        Event::new(event_time, lineage_reference, event_type)
    }
}

#[contract_trait]
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
    > GillespieEventSampler<H, D, R, S, ConditionalCoalescenceSampler<H, R, S>>
    for ConditionalGillespieEventSampler<H, D, R, S>
{
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, D, R, S, ConditionalCoalescenceSampler<H, R, S>>,
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

#[allow(clippy::module_name_repetitions)]
pub struct NoSelfDispersalConditionalNoCoalescenceEventTypeSampler;

use super::EventTypeSampler;

use necsim_core::event_generator::EventType;
use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::coalescence_sampler::ConditionalCoalescenceSampler;

#[contract_trait]
impl<L: LineageReference> EventTypeSampler<L>
    for NoSelfDispersalConditionalNoCoalescenceEventTypeSampler
{
    #[debug_ensures(match &ret {
        EventType::Speciation => true,
        EventType::Dispersal {
            origin,
            target,
            coalescence,
        } => coalescence.is_some() == (origin == target),
    }, "always coalesce on self-dispersal, never coalesce on out-dispersal")]
    fn sample_event_type_at_location(
        &self,
        location: &Location,
        settings: &SimulationSettings<impl Landscape>,
        coalescence_sampler: &impl ConditionalCoalescenceSampler<L>,
        rng: &mut impl Rng,
    ) -> EventType<L> {
        let speciation = settings.speciation_probability_per_generation();

        let self_dispersal = 0.5_f64; // TODO
        let out_dispersal = 1.0_f64 - self_dispersal;

        let habitat_at_location = settings.landscape().get_habitat_at_location(location);
        let coalescence_at_location = coalescence_sampler
            .get_coalescence_probability_at_location(location, habitat_at_location);

        let total_probability = speciation
            + (1.0_f64 - speciation) * (out_dispersal + self_dispersal * coalescence_at_location);

        let sample = rng.sample_uniform() * total_probability;

        if sample < speciation {
            return EventType::Speciation;
        }

        let dispersal_origin = location;

        if sample < (speciation + (1.0_f64 - speciation) * self_dispersal * coalescence_at_location)
        {
            return EventType::Dispersal {
                origin: dispersal_origin.clone(),
                target: dispersal_origin.clone(),
                coalescence: Some(
                    coalescence_sampler.sample_coalescence_at_location(location, rng),
                ),
            };
        }

        // TODO: How are we sure this does not perform self-dispersal?
        let dispersal_target = settings
            .landscape()
            .sample_dispersal_from_location(&dispersal_origin, rng);

        EventType::Dispersal {
            origin: dispersal_origin.clone(),
            target: dispersal_target,
            coalescence: coalescence_sampler.sample_optional_coalescence_at_location(
                location,
                habitat_at_location,
                rng,
            ),
        }
    }
}

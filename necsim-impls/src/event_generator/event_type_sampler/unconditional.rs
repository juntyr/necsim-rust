#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalEventTypeSampler;

use super::EventTypeSampler;

use necsim_core::event_generator::EventType;
use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::coalescence_sampler::CoalescenceSampler;

#[contract_trait]
impl<L: LineageReference> EventTypeSampler<L> for UnconditionalEventTypeSampler {
    #[must_use]
    fn sample_event_type_at_location(
        &self,
        location: &Location,
        settings: &SimulationSettings<impl Landscape>,
        coalescence_sampler: &impl CoalescenceSampler<L>,
        rng: &mut impl Rng,
    ) -> EventType<L> {
        if rng.sample_event(settings.speciation_probability_per_generation()) {
            return EventType::Speciation;
        }

        let dispersal_origin = location;
        let dispersal_target = settings
            .landscape()
            .sample_dispersal_from_location(&dispersal_origin, rng);

        EventType::Dispersal {
            origin: dispersal_origin.clone(),
            target: dispersal_target,
            coalescence: coalescence_sampler.sample_optional_coalescence_at_location(
                location,
                settings.landscape().get_habitat_at_location(location),
                rng,
            ),
        }
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalNoCoalescenceEventTypeSampler;

use super::EventTypeSampler;

use necsim_core::event_generator::EventType;
use necsim_core::landscape::{Landscape, Location};
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

impl EventTypeSampler for UnconditionalNoCoalescenceEventTypeSampler {
    fn sample_event_type_at_location(
        &self,
        location: &Location,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> EventType {
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
            coalescence: false,
        }
    }
}

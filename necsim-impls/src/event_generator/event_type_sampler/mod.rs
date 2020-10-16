pub mod unconditional_no_coalescence;

use necsim_core::event_generator::EventType;
use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

pub trait EventTypeSampler<L: LineageReference> {
    fn sample_event_type_at_location(
        &self,
        location: &Location,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> EventType<L>;
}

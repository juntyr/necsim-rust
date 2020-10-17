pub mod unconditional_no_coalescence;

use necsim_core::event_generator::EventType;
use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventTypeSampler<L: LineageReference> {
    #[debug_requires(
        settings.landscape().get_habitat_at_location(location) > 0,
        "location is habitable event origin"
    )]
    fn sample_event_type_at_location(
        &self,
        location: &Location,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> EventType<L>;
}

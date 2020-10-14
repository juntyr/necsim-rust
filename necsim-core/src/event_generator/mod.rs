mod event;

pub use event::{Event, EventType};

use crate::landscape::Landscape;
use crate::rng::Rng;
use crate::simulation::SimulationSettings;

pub trait EventGenerator {
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event>;
}

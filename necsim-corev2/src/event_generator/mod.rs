mod event;

pub use event::{Event, EventType};

use crate::landscape::Landscape;
use crate::lineage::LineageReference;
use crate::rng::Rng;
use crate::simulation::SimulationSettings;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventGenerator<L: LineageReference> {
    #[debug_requires(time >= 0.0_f64, "time >= 0.0")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().time() > time,
        "event occurs later than time"
    )]
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event<L>>;
}

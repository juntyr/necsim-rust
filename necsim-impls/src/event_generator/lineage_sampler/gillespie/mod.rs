use std::cmp::Ordering;
use std::ops::Index;

use priority_queue::PriorityQueue;

use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::Lineage;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::lineage_sampler::global_store::{GlobalLineageStore, LineageReference};

mod coalescence;
mod sampler;

#[derive(PartialEq, Copy, Clone)]
struct EventTime(f64);

impl Eq for EventTime {}

impl PartialOrd for EventTime {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        other.0.partial_cmp(&self.0)
    }
}

impl Ord for EventTime {
    fn cmp(&self, other: &Self) -> Ordering {
        crate::f64::total_cmp_f64(other.0, self.0)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct GillespieLineageSampler {
    lineages_store: GlobalLineageStore,
    active_locations: PriorityQueue<Location, EventTime>,
    number_active_lineages: usize,
}

impl GillespieLineageSampler {
    #[must_use]
    pub fn new(settings: &SimulationSettings<impl Landscape>, rng: &mut impl Rng) -> Self {
        let lineages_store = GlobalLineageStore::new(settings);

        let landscape_extent = settings.landscape().get_extent();

        let mut active_locations: Vec<(Location, EventTime)> = Vec::with_capacity(
            landscape_extent.width() as usize * landscape_extent.height() as usize,
        );

        let mut number_active_lineages: usize = 0;

        let landscape_extent = settings.landscape().get_extent();

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                let location = Location::new(x, y);

                let number_active_lineages_at_location = lineages_store
                    .get_active_lineages_at_location(&location)
                    .len();

                number_active_lineages += number_active_lineages_at_location;

                if number_active_lineages_at_location > 0 {
                    #[allow(clippy::cast_precision_loss)]
                    let lambda = 0.5_f64 * number_active_lineages_at_location as f64;

                    active_locations.push((location, EventTime(rng.sample_exponential(lambda))));
                }
            }
        }

        active_locations.shrink_to_fit();

        Self {
            lineages_store,
            active_locations: PriorityQueue::from(active_locations),
            number_active_lineages,
        }
    }
}

impl Index<LineageReference> for GillespieLineageSampler {
    type Output = Lineage;

    #[must_use]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference]
    }
}

use std::cmp::Ordering;
use std::ops::Index;

use array2d::Array2D;
use priority_queue::PriorityQueue;

use necsim_core::landscape::{Landscape, LandscapeExtent, Location};
use necsim_core::lineage::Lineage;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

mod coalescence;
mod contract;
mod r#impl;
mod store;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LineageReference(usize);

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

impl necsim_core::lineage::LineageReference for LineageReference {}

#[allow(clippy::module_name_repetitions)]
pub struct GlobalGillespieStore {
    landscape_extent: LandscapeExtent,
    lineages_store: Vec<Lineage>,
    active_locations: PriorityQueue<Location, EventTime>,
    location_to_lineage_references: Array2D<Vec<LineageReference>>,
    number_active_lineages: usize,
}

impl GlobalGillespieStore {
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_ensures(
        ret.landscape_extent == settings.landscape().get_extent(),
        "stores landscape_extent"
    )]
    #[debug_ensures(if settings.sample_percentage() == 0.0_f64 {
        ret.number_active_lineages() == 0
    } else if settings.sample_percentage() == 1.0_f64 {
        ret.number_active_lineages() == settings.landscape().get_total_habitat()
    } else {
        true
    }, "samples active lineages according to settings.sample_percentage()")]
    pub fn new(settings: &SimulationSettings<impl Landscape>, rng: &mut impl Rng) -> Self {
        let landscape = settings.landscape();
        let sample_percentage = settings.sample_percentage();

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        #[allow(clippy::cast_precision_loss)]
        let mut lineages_store = Vec::with_capacity(
            ((landscape.get_total_habitat() as f64) * sample_percentage) as usize,
        );

        let landscape_extent = landscape.get_extent();

        let mut location_to_lineage_references = Array2D::filled_with(
            Vec::new(),
            landscape_extent.height() as usize,
            landscape_extent.width() as usize,
        );

        let mut active_locations: Vec<(Location, EventTime)> = Vec::with_capacity(
            landscape_extent.width() as usize * landscape_extent.height() as usize,
        );

        let x_from = landscape_extent.x();
        let y_from = landscape_extent.y();

        for y_offset in 0..landscape_extent.height() {
            for x_offset in 0..landscape_extent.width() {
                let location = Location::new(x_from + x_offset, y_from + y_offset);

                let lineages_at_location =
                    &mut location_to_lineage_references[(y_offset as usize, x_offset as usize)];

                #[allow(clippy::cast_possible_truncation)]
                #[allow(clippy::cast_sign_loss)]
                let sampled_habitat_at_location =
                    (f64::from(landscape.get_habitat_at_location(&location)) * sample_percentage)
                        .floor() as usize;

                for _ in 0..sampled_habitat_at_location {
                    let lineage_reference = LineageReference(lineages_store.len());
                    let index_at_location = lineages_at_location.len();

                    lineages_at_location.push(lineage_reference);
                    lineages_store.push(Lineage::new(location.clone(), index_at_location));
                }

                if !lineages_at_location.is_empty() {
                    #[allow(clippy::cast_precision_loss)]
                    let lambda = 0.5_f64 * lineages_at_location.len() as f64;

                    active_locations.push((location, EventTime(rng.sample_exponential(lambda))));
                }
            }
        }

        lineages_store.shrink_to_fit();
        active_locations.shrink_to_fit();

        Self {
            landscape_extent,
            number_active_lineages: lineages_store.len(),
            lineages_store,
            active_locations: PriorityQueue::from(active_locations),
            location_to_lineage_references,
        }
    }

    #[must_use]
    pub fn number_active_lineages(&self) -> usize {
        self.number_active_lineages
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    pub fn get_number_active_lineages_at_location(&self, location: &Location) -> usize {
        self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )]
            .len()
    }
}

impl Index<LineageReference> for GlobalGillespieStore {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(reference.0 < self.lineages_store.len(), "lineage reference is in range")]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}

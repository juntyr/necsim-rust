use std::ops::Index;

use array2d::Array2D;

use necsim_core::landscape::{Landscape, LandscapeExtent, Location};
use necsim_core::lineage::Lineage;
use necsim_core::simulation::SimulationSettings;

mod coalescence;
mod contract;
mod update;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LineageReference(usize);

impl necsim_core::lineage::LineageReference for LineageReference {}

pub struct GlobalLineageStore {
    landscape_extent: LandscapeExtent,
    lineages_store: Vec<Lineage>,
    location_to_lineage_references: Array2D<Vec<LineageReference>>,
}

impl GlobalLineageStore {
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_ensures(
        ret.landscape_extent == settings.landscape().get_extent(),
        "stores landscape_extent"
    )]
    #[debug_ensures(if settings.sample_percentage() == 0.0_f64 {
        ret.lineages_store.is_empty()
    } else if settings.sample_percentage() == 1.0_f64 {
        ret.lineages_store.len() == settings.landscape().get_total_habitat()
    } else {
        true
    }, "samples active lineages according to settings.sample_percentage()")]
    pub fn new(settings: &SimulationSettings<impl Landscape>) -> Self {
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
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent,
            lineages_store,
            location_to_lineage_references,
        }
    }

    #[must_use]
    #[debug_requires(
        self.landscape_extent.contains(location),
        "location is inside landscape extent"
    )]
    pub fn get_active_lineages_at_location(&self, location: &Location) -> &[LineageReference] {
        &self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )]
    }
}

impl Index<LineageReference> for GlobalLineageStore {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(reference.0 < self.lineages_store.len(), "lineage reference is in range")]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}

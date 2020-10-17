use std::ops::Index;

use array2d::Array2D;

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

impl necsim_core::lineage::LineageReference for LineageReference {}

pub struct GlobalLineageStore {
    landscape_extent: LandscapeExtent,
    lineages_store: Vec<Lineage>,
    active_lineage_references: Vec<LineageReference>,
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
        #[allow(clippy::cast_lossless)]
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

                for index_at_location in 0..landscape.get_habitat_at_location(&location) {
                    #[allow(clippy::float_cmp)]
                    if sample_percentage == 1.0 || rng.sample_event(sample_percentage) {
                        lineages_at_location.push(LineageReference(lineages_store.len()));
                        lineages_store
                            .push(Lineage::new(location.clone(), index_at_location as usize));
                    }
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            landscape_extent,
            active_lineage_references: (0..lineages_store.len()).map(LineageReference).collect(),
            lineages_store,
            location_to_lineage_references,
        }
    }

    #[must_use]
    pub fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
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

impl Index<LineageReference> for GlobalLineageStore {
    type Output = Lineage;

    #[must_use]
    #[debug_requires(reference.0 < self.lineages_store.len(), "lineage reference is in range")]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}

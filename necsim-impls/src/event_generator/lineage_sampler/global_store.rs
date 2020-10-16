use std::ops::Index;

use array2d::Array2D;

use necsim_core::landscape::{Landscape, Location};
use necsim_core::lineage::Lineage;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct LineageReference(usize);

impl necsim_core::lineage::LineageReference for LineageReference {}

pub struct GlobalLineageStore {
    lineages_store: Vec<Lineage>,
    active_lineage_references: Vec<LineageReference>,
    location_to_lineage_references: Array2D<Vec<LineageReference>>,
}

impl GlobalLineageStore {
    #[must_use]
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

        for y in landscape_extent.y()..(landscape_extent.y() + landscape_extent.height()) {
            for x in landscape_extent.x()..(landscape_extent.x() + landscape_extent.width()) {
                let location = Location::new(x, y);

                let lineages_at_location =
                    &mut location_to_lineage_references[(y as usize, x as usize)];

                for index_at_location in 0..landscape.get_habitat_at_location(&location) {
                    if (sample_percentage - 1.0_f64).abs() < f64::EPSILON
                        || rng.sample_event(sample_percentage)
                    {
                        lineages_at_location.push(LineageReference(lineages_store.len()));
                        lineages_store
                            .push(Lineage::new(location.clone(), index_at_location as usize));
                    }
                }
            }
        }

        lineages_store.shrink_to_fit();

        Self {
            active_lineage_references: (0..lineages_store.len()).map(LineageReference).collect(),
            lineages_store,
            location_to_lineage_references,
        }
    }

    fn add_lineage_to_location(&mut self, reference: LineageReference, location: Location) {
        let lineages_at_location = &mut self.location_to_lineage_references
            [(location.y() as usize, location.x() as usize)];

        // TODO: We should assert that we never surpass the available habitat

        lineages_at_location.push(reference);

        unsafe {
            self.lineages_store[reference.0].move_to_location(location, lineages_at_location.len())
        };
    }

    fn remove_lineage_from_its_location(&mut self, reference: LineageReference) {
        let lineage = &self.lineages_store[reference.0];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            lineage.location().y() as usize,
            lineage.location().x() as usize,
        )];

        if let Some(last_lineage_at_location) = lineages_at_location.pop() {
            let lineage_index_at_location = lineage.index_at_location();

            if lineage_index_at_location < lineages_at_location.len() {
                lineages_at_location[lineage_index_at_location] = last_lineage_at_location;

                unsafe {
                    self.lineages_store[last_lineage_at_location.0]
                        .update_index_at_location(lineage_index_at_location)
                };
            }
        }
    }

    #[must_use]
    pub fn pop_random_active_lineage_reference(
        &mut self,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        let last_active_lineage_reference = match self.active_lineage_references.pop() {
            Some(reference) => reference,
            None => return None,
        };

        let chosen_active_lineage_index =
            rng.sample_index(self.active_lineage_references.len() + 1);

        let chosen_lineage_reference =
            if chosen_active_lineage_index == self.active_lineage_references.len() {
                last_active_lineage_reference
            } else {
                let chosen_lineage_reference =
                    self.active_lineage_references[chosen_active_lineage_index];

                self.active_lineage_references[chosen_active_lineage_index] =
                    last_active_lineage_reference;

                chosen_lineage_reference
            };

        self.remove_lineage_from_its_location(chosen_lineage_reference);

        Some(chosen_lineage_reference)
    }

    pub fn push_active_lineage_reference_at_location(
        &mut self,
        reference: LineageReference,
        location: Location,
    ) {
        self.add_lineage_to_location(reference, location);

        self.active_lineage_references.push(reference);
    }

    #[must_use]
    pub fn number_active_lineages(&self) -> usize {
        self.active_lineage_references.len()
    }

    #[must_use]
    pub fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<LineageReference> {
        let population = self.get_number_active_lineages_at_location(location);

        let chosen_coalescence = rng.sample_index(habitat as usize);

        if chosen_coalescence >= population {
            return None;
        }

        Some(
            self.location_to_lineage_references[(location.y() as usize, location.x() as usize)]
                [chosen_coalescence],
        )
    }

    #[must_use]
    pub fn get_number_active_lineages_at_location(&self, location: &Location) -> usize {
        self.location_to_lineage_references[(location.y() as usize, location.x() as usize)].len()
    }
}

impl Index<LineageReference> for GlobalLineageStore {
    type Output = Lineage;

    #[must_use]
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}

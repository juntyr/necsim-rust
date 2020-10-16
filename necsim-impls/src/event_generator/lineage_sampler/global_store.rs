use std::ops::Index;

use array2d::Array2D;

use necsim_core::landscape::{Landscape, LandscapeExtent, Location};
use necsim_core::lineage::Lineage;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

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
    #[debug_ensures(ret.landscape_extent == settings.landscape().get_extent())]
    #[debug_ensures(if settings.sample_percentage() == 0.0_f64 {
        ret.number_active_lineages() == 0
    } else if settings.sample_percentage() == 1.0_f64 {
        ret.number_active_lineages() == settings.landscape().get_total_habitat()
    } else {
        true
    })]
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
            landscape_extent,
            active_lineage_references: (0..lineages_store.len()).map(LineageReference).collect(),
            lineages_store,
            location_to_lineage_references,
        }
    }

    #[debug_requires(reference.0 < self.lineages_store.len())]
    #[debug_requires(
        location.x() >= self.landscape_extent.x() &&
        location.x() < self.landscape_extent.x() + self.landscape_extent.width() &&
        location.y() >= self.landscape_extent.y() &&
        location.y() < self.landscape_extent.y() + self.landscape_extent.height()
    )]
    // TODO: Check that the lineage was added to the correct location
    // TODO: Check that all lineages at the position point to themselves
    fn add_lineage_to_location(&mut self, reference: LineageReference, location: Location) {
        let lineages_at_location = &mut self.location_to_lineage_references[(
            (location.y() - self.landscape_extent.y()) as usize,
            (location.x() - self.landscape_extent.x()) as usize,
        )];

        // TODO: We should be able to assert that we never surpass the available habitat

        lineages_at_location.push(reference);

        unsafe {
            self.lineages_store[reference.0]
                .move_to_location(location, lineages_at_location.len() - 1)
        };
    }

    #[debug_requires(reference.0 < self.lineages_store.len())]
    #[debug_requires(
        self[reference].location().x() >= self.landscape_extent.x() &&
        self[reference].location().x() < (
            self.landscape_extent.x() + self.landscape_extent.width()
        ) &&
        self[reference].location().y() >= self.landscape_extent.y() &&
        self[reference].location().y() < (
            self.landscape_extent.y() + self.landscape_extent.height()
        )
    )]
    #[debug_requires({
        let lineage = &self[reference];

        let lineages_at_location = &self.location_to_lineage_references[(
            (lineage.location().y() - self.landscape_extent.y()) as usize,
            (lineage.location().x() - self.landscape_extent.x()) as usize,
        )];

        lineages_at_location[lineage.index_at_location()] == reference
    })]
    // TODO: Check that the lineage was removed
    // TODO: Check that all lineages at the position point to themselves
    fn remove_lineage_from_its_location(&mut self, reference: LineageReference) {
        let lineage = &self.lineages_store[reference.0];

        let lineages_at_location = &mut self.location_to_lineage_references[(
            (lineage.location().y() - self.landscape_extent.y()) as usize,
            (lineage.location().x() - self.landscape_extent.x()) as usize,
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
    // TODO: Check that the number of active lineages has been decremented iff Some
    // TODO: Check that returned reference is no longer active and not at its location
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

    // TODO: Check that the lineage is not active and not at its location
    // TODO: Check that the lineage has been added to its location
    // TODO: Check that the lineage is an active lineage again
    // TODO: Check that the number of active lineages has been incremented
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
    #[debug_requires(
        location.x() >= self.landscape_extent.x() &&
        location.x() < self.landscape_extent.x() + self.landscape_extent.width() &&
        location.y() >= self.landscape_extent.y() &&
        location.y() < self.landscape_extent.y() + self.landscape_extent.height()
    )]
    #[debug_requires(habitat > 0)]
    // TODO: Check that iff lineage is returned it is active and at the location
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
            self.location_to_lineage_references[(
                (location.y() - self.landscape_extent.y()) as usize,
                (location.x() - self.landscape_extent.x()) as usize,
            )][chosen_coalescence],
        )
    }

    #[must_use]
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
    fn index(&self, reference: LineageReference) -> &Self::Output {
        &self.lineages_store[reference.0]
    }
}

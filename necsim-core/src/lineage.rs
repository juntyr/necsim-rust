use crate::landscape::Location;

pub struct Lineage {
    location: Location,
    index_at_location: usize,
}

impl Lineage {
    #[must_use]
    pub fn new(location: Location, index_at_location: usize) -> Self {
        Self {
            location,
            index_at_location,
        }
    }

    #[must_use]
    pub fn location(&self) -> &Location {
        &self.location
    }

    #[must_use]
    pub fn index_at_location(&self) -> usize {
        self.index_at_location
    }

    /// # Safety
    /// This method should only be called by internal `EventGenerator` code to update the
    /// state of the lineages being simulated.
    pub unsafe fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = location;

        self.update_index_at_location(index_at_location);
    }

    /// # Safety
    /// This method should only be called by internal `EventGenerator` code to update the
    /// state of the lineages being simulated.
    pub unsafe fn update_index_at_location(&mut self, index_at_location: usize) {
        self.index_at_location = index_at_location;
    }
}

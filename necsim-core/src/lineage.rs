use crate::landscape::Location;

pub struct Lineage {
    location: Location,
    index_at_location: usize,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(
        ret.location() == &old(location.clone()) &&
        ret.index_at_location() == index_at_location
    )]
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
    #[debug_ensures(
        self.location() == &old(location.clone()) &&
        self.index_at_location() == index_at_location
    )]
    pub unsafe fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = location;

        self.update_index_at_location(index_at_location);
    }

    /// # Safety
    /// This method should only be called by internal `EventGenerator` code to update the
    /// state of the lineages being simulated.
    #[debug_ensures(self.index_at_location() == index_at_location)]
    pub unsafe fn update_index_at_location(&mut self, index_at_location: usize) {
        self.index_at_location = index_at_location;
    }
}

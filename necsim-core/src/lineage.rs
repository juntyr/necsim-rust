use std::hash::Hash;

use crate::landscape::Location;

#[allow(clippy::module_name_repetitions)]
pub trait LineageReference: PartialEq + Eq + Hash {}

pub struct Lineage {
    location: Location,
    index_at_location: usize,
}

impl Lineage {
    #[must_use]
    #[debug_ensures(ret.location() == &old(location.clone()), "stores the location")]
    #[debug_ensures(ret.index_at_location() == index_at_location, "stores the index_at_location")]
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
    #[debug_ensures(self.location() == &old(location.clone()), "updates the location")]
    #[debug_ensures(self.index_at_location() == index_at_location, "updates the index_at_location")]
    pub unsafe fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = location;

        self.update_index_at_location(index_at_location);
    }

    /// # Safety
    /// This method should only be called by internal `EventGenerator` code to update the
    /// state of the lineages being simulated.
    #[debug_ensures(self.index_at_location() == index_at_location, "updates the index_at_location")]
    pub unsafe fn update_index_at_location(&mut self, index_at_location: usize) {
        self.index_at_location = index_at_location;
    }
}

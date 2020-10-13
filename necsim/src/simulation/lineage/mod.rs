mod lineages;
mod reference;

pub use lineages::SimulationLineages;
pub use reference::LineageReference;

use crate::landscape::Location;

pub struct Lineage {
    location: Location,
    index_at_location: usize,
}

impl Lineage {
    #[must_use]
    /*pub(self)*/
    fn new(location: Location, index_at_location: usize) -> Self {
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

    /*pub(self)*/
    fn move_to_location(&mut self, location: Location, index_at_location: usize) {
        self.location = location;

        self.update_index_at_location(index_at_location);
    }

    /*pub(self)*/
    fn update_index_at_location(&mut self, index_at_location: usize) {
        self.index_at_location = index_at_location;
    }
}

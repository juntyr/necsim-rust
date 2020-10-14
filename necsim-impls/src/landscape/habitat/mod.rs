pub mod in_memory;

use necsim_core::landscape::{LandscapeExtent, Location};

pub trait Habitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent;

    #[must_use]
    fn get_total_habitat(&self) -> u32;
    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32;
}

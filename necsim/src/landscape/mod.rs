mod extent;
mod location;

pub use extent::LandscapeExtent;
pub use location::Location;

pub mod impls;

use crate::rng;

pub trait Landscape {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent;

    #[must_use]
    fn get_total_habitat(&self) -> u32;
    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32;

    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        rng: &mut impl rng::Rng,
    ) -> Location;
}

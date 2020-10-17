mod extent;
mod location;

#[macro_use]
pub mod contract;

pub use extent::LandscapeExtent;
pub use location::Location;

use crate::rng::Rng;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Landscape {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent;

    #[must_use]
    #[debug_ensures(
        ret == explicit_landscape_total_habitat_contract!(self),
        "total habitat is the sum of all habitat in the extent of the landscape"
    )]
    fn get_total_habitat(&self) -> usize;

    #[must_use]
    #[debug_requires(self.get_extent().contains(location), "location is inside landscape extent")]
    fn get_habitat_at_location(&self, location: &Location) -> u32;

    #[must_use]
    #[debug_requires(self.get_habitat_at_location(location) > 0, "location is habitable origin")]
    #[debug_ensures(self.get_habitat_at_location(&ret) > 0, "destination is habitable")]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location;
}

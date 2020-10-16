mod extent;
mod location;

pub use extent::LandscapeExtent;
pub use location::Location;

use crate::rng::Rng;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Landscape {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent;

    #[must_use]
    #[debug_ensures(ret == {
        let extent = self.get_extent();

        let mut total_habitat: usize = 0;

        for y in extent.y()..(extent.y() + extent.height()) {
            for x in extent.x()..(extent.x() + extent.width()) {
                total_habitat += self.get_habitat_at_location(
                    &Location::new(x, y)
                ) as usize;
            }
        }

        total_habitat
    })]
    fn get_total_habitat(&self) -> usize;
    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32;

    #[must_use]
    #[debug_requires(self.get_habitat_at_location(location) > 0)]
    #[debug_ensures(self.get_habitat_at_location(&ret) > 0)]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location;
}

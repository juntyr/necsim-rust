use crate::landscape::{IndexedLocation, LandscapeExtent, Location};

use super::F64Core;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Habitat<F: F64Core>: crate::cogs::Backup + core::fmt::Debug + Sized {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent;

    #[must_use]
    fn contains(&self, location: &Location) -> bool {
        self.get_extent().contains(location)
    }

    #[must_use]
    #[debug_ensures(ret == {
        let extent = self.get_extent();

        let mut total_habitat: u64 = 0;

        for y in extent.y()..(extent.y() + extent.height()) {
            for x in extent.x()..(extent.x() + extent.width()) {
                total_habitat += u64::from(self.get_habitat_at_location(&Location::new(x, y)));
            }
        }

        total_habitat
    }, "total habitat is the sum of all habitat in the extent of the habitat")]
    fn get_total_habitat(&self) -> u64;

    #[must_use]
    #[debug_requires(self.get_extent().contains(location), "location is inside habitat extent")]
    fn get_habitat_at_location(&self, location: &Location) -> u32;

    #[must_use]
    #[debug_requires(
        self.get_extent().contains(indexed_location.location()),
        "location is inside habitat extent"
    )]
    #[debug_requires(
        indexed_location.index() < self.get_habitat_at_location(indexed_location.location()),
        "index is within the location's habitat capacity"
    )]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64;
}

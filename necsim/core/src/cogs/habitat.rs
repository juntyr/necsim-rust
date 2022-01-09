use crate::landscape::{IndexedLocation, LandscapeExtent, Location};

use super::MathsCore;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Habitat<M: MathsCore>: crate::cogs::Backup + core::fmt::Debug + Sized {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent;

    #[must_use]
    fn contains(&self, location: &Location) -> bool {
        self.get_extent().contains(location)
    }

    #[must_use]
    #[debug_ensures(ret == {
        self.get_extent()
            .iter()
            .map(|location| u64::from(self.get_habitat_at_location(&location)))
            .sum()
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

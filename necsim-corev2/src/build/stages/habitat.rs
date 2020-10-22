use crate::landscape::LandscapeExtent;
use crate::landscape::Location;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Habitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent;

    #[must_use]
    #[debug_ensures(
        ret == explicit_landscape_total_habitat_contract!(self),
        "total habitat is the sum of all habitat in the extent of the habitat"
    )]
    fn get_total_habitat(&self) -> usize;

    #[must_use]
    #[debug_requires(self.get_extent().contains(location), "location is inside habitat extent")]
    fn get_habitat_at_location(&self, location: &Location) -> u32;
}

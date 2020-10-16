pub mod in_memory;

use necsim_core::landscape::{LandscapeExtent, Location};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Habitat {
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
    #[debug_requires(
        location.x() >= self.get_extent().x() &&
        location.x() < self.get_extent().x() + self.get_extent().width() &&
        location.y() >= self.get_extent().y() &&
        location.y() < self.get_extent().y() + self.get_extent().height()
    )]
    fn get_habitat_at_location(&self, location: &Location) -> u32;
}

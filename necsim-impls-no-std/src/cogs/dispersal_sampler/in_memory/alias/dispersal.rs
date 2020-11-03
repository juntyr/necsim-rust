use core::ops::Range;

use necsim_core::cogs::{DispersalSampler, Habitat};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use crate::alias::AliasMethodSamplerAtom;

use super::InMemoryAliasDispersalSampler;

impl<H: Habitat> DispersalSampler<H> for InMemoryAliasDispersalSampler {
    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    #[debug_ensures(self.habitat_extent.contains(&ret), "target is inside habitat extent")]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        let location_index = (
            (location.y() - self.habitat_extent.y()) as usize,
            (location.x() - self.habitat_extent.x()) as usize,
        );

        let alias_dispersals_at_location = &self.alias_dispersal_buffer
            [Into::<Range<usize>>::into(self.alias_dispersal_ranges[location_index].clone())];

        let dispersal_target_index: usize =
            AliasMethodSamplerAtom::sample_event(alias_dispersals_at_location, rng);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
        )
    }
}

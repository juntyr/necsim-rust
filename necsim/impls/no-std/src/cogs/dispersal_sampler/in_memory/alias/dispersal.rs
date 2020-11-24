use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore},
    landscape::Location,
};

use super::InMemoryAliasDispersalSampler;

impl<H: Habitat, G: RngCore> DispersalSampler<H, G> for InMemoryAliasDispersalSampler<H, G> {
    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    #[debug_ensures(self.habitat_extent.contains(&ret), "target is inside habitat extent")]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut G) -> Location {
        let alias_dispersal_at_location = self.alias_dispersal[(
            (location.y() - self.habitat_extent.y()) as usize,
            (location.x() - self.habitat_extent.x()) as usize,
        )]
            .as_ref()
            .expect("habitat dispersal origin must disperse somewhere");

        let dispersal_target_index = alias_dispersal_at_location.sample_event(rng);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
        )
    }
}

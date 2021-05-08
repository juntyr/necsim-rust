use core::ops::Range;

use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore},
    landscape::Location,
};

use crate::alias::packed::AliasMethodSamplerAtom;

use super::InMemoryPackedAliasDispersalSampler;

#[contract_trait]
impl<H: Habitat, G: RngCore> DispersalSampler<H, G> for InMemoryPackedAliasDispersalSampler<H, G> {
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let location_row = (location.y() - habitat.get_extent().y()) as usize;
        let location_column = (location.x() - habitat.get_extent().x()) as usize;

        // Only safe as trait precondition that `location` is inside `habitat`
        let alias_range = unsafe {
            Range::<usize>::from(
                self.alias_dispersal_ranges
                    .get(location_row, location_column)
                    .unwrap_unchecked()
                    .clone(),
            )
        };

        // Safe by the construction of `InMemoryPackedAliasDispersalSampler`
        let alias_dispersals_at_location =
            unsafe { &self.alias_dispersal_buffer.get_unchecked(alias_range) };

        let dispersal_target_index: usize =
            AliasMethodSamplerAtom::sample_event(alias_dispersals_at_location, rng);

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }
}

use core::ops::Range;

use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize},
        DispersalSampler, DistributionSampler, Habitat, MathsCore, Rng,
    },
    landscape::Location,
};

use crate::alias::packed::AliasMethodSamplerAtom;

use super::InMemoryPackedAliasDispersalSampler;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> DispersalSampler<M, H, G>
    for InMemoryPackedAliasDispersalSampler<M, H, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>,
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let location_row = location.y().wrapping_sub(habitat.get_extent().y()) as usize;
        let location_column = location.x().wrapping_sub(habitat.get_extent().x()) as usize;

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
            habitat.get_extent().x().wrapping_add(
                (dispersal_target_index % usize::from(habitat.get_extent().width())) as u32,
            ),
            habitat.get_extent().y().wrapping_add(
                (dispersal_target_index / usize::from(habitat.get_extent().width())) as u32,
            ),
        )
    }
}

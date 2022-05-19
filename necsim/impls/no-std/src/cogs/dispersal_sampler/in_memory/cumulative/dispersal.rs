use necsim_core::{
    cogs::{
        distribution::UniformClosedOpenUnit, DispersalSampler, DistributionSampler, Habitat,
        MathsCore, Rng, SampledDistribution,
    },
    landscape::Location,
};

use super::InMemoryCumulativeDispersalSampler;

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> DispersalSampler<M, H, G>
    for InMemoryCumulativeDispersalSampler<M, H, G>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>,
{
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        let location_index = (location.y().wrapping_sub(habitat.get_extent().y()) as usize)
            * usize::from(habitat.get_extent().width())
            + (location.x().wrapping_sub(habitat.get_extent().x()) as usize);

        let habitat_area =
            usize::from(habitat.get_extent().width()) * usize::from(habitat.get_extent().height());

        let cumulative_dispersals_at_location = &self.cumulative_dispersal
            [location_index * habitat_area..(location_index + 1) * habitat_area];

        let cumulative_percentage_sample = UniformClosedOpenUnit::sample(rng).into();

        let dispersal_target_index = usize::min(
            match cumulative_dispersals_at_location.binary_search(&cumulative_percentage_sample) {
                Ok(index) | Err(index) => index,
            },
            habitat_area - 1,
        );

        // Sampling the cumulative probability table using binary search can return
        // non-habitat locations. We correct for this by storing the index of the
        // last valid habitat (the alias method will make this obsolete).
        let Some(Some(valid_dispersal_target_index)) = self
            .valid_dispersal_targets
            .get(location_index * habitat_area + dispersal_target_index).copied()
        else {
            unreachable!("habitat dispersal origin must disperse somewhere")
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            habitat.get_extent().x().wrapping_add(
                (valid_dispersal_target_index % usize::from(habitat.get_extent().width())) as u32,
            ),
            habitat.get_extent().y().wrapping_add(
                (valid_dispersal_target_index / usize::from(habitat.get_extent().width())) as u32,
            ),
        )
    }
}

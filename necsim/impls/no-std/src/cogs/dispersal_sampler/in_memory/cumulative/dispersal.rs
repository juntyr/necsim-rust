use necsim_core::{
    cogs::{DispersalSampler, Habitat, RngCore},
    landscape::Location,
};

use super::InMemoryCumulativeDispersalSampler;

#[contract_trait]
impl<H: Habitat, G: RngCore> DispersalSampler<H, G> for InMemoryCumulativeDispersalSampler {
    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location {
        use necsim_core::cogs::RngSampler;

        let location_index = ((location.y() - habitat.get_extent().y()) as usize)
            * (habitat.get_extent().width() as usize)
            + ((location.x() - habitat.get_extent().x()) as usize);

        let habitat_area =
            (habitat.get_extent().width() as usize) * (habitat.get_extent().height() as usize);

        let cumulative_dispersals_at_location = &self.cumulative_dispersal
            [location_index * habitat_area..(location_index + 1) * habitat_area];

        let cumulative_percentage_sample = rng.sample_uniform();

        let dispersal_target_index = usize::min(
            match cumulative_dispersals_at_location
                .binary_search_by(|v| v.total_cmp(&cumulative_percentage_sample))
            {
                Ok(index) | Err(index) => index,
            },
            habitat_area - 1,
        );

        // Sampling the cumulative probability table using binary search can return
        // non-habitat locations. We correct for this by storing the index of the
        // last valid habitat (the alias method will make this obsolete).
        #[allow(clippy::match_on_vec_items)]
        let valid_dispersal_target_index = match self.valid_dispersal_targets
            [location_index * habitat_area + dispersal_target_index]
        {
            Some(valid_dispersal_target_index) => valid_dispersal_target_index,
            None => unreachable!("habitat dispersal origin must disperse somewhere"),
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (valid_dispersal_target_index % (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().x(),
            (valid_dispersal_target_index / (habitat.get_extent().width() as usize)) as u32
                + habitat.get_extent().y(),
        )
    }
}

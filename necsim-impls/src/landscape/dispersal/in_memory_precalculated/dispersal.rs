use std::cmp::Ordering;

use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use crate::landscape::dispersal::Dispersal;

use super::InMemoryPrecalculatedDispersal;

impl Dispersal for InMemoryPrecalculatedDispersal {
    #[must_use]
    #[debug_requires(self.habitat_extent.contains(location), "location is inside habitat extent")]
    #[debug_ensures(self.habitat_extent.contains(&ret), "target is inside habitat extent")]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        let location_index = ((location.y() - self.habitat_extent.y()) as usize)
            * (self.habitat_extent.width() as usize)
            + ((location.x() - self.habitat_extent.x()) as usize);

        let habitat_area =
            (self.habitat_extent.width() as usize) * (self.habitat_extent.height() as usize);

        let cumulative_dispersals_at_location = &self.cumulative_dispersal
            [location_index * habitat_area..(location_index + 1) * habitat_area];

        let cumulative_percentage_sample = rng.sample_uniform();

        let dispersal_target_index = usize::min(
            match cumulative_dispersals_at_location.binary_search_by(|v| {
                v.partial_cmp(&cumulative_percentage_sample)
                    .unwrap_or(Ordering::Equal)
            }) {
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
            (valid_dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (valid_dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
        )
    }
}

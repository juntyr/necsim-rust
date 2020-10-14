use std::cmp::Ordering;

use array2d::Array2D;
use thiserror::Error;

use necsim_core::landscape::{LandscapeExtent, Location};
use necsim_core::rng::Rng;

use super::Dispersal;

#[derive(Error, Debug)]
#[error("The size of the dispersal map was inconsistent with the size of the habitat map.")]
pub struct InconsistentDispersalMapSize;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryPrecalculatedDispersal {
    cumulative_dispersal: Vec<f64>,
    habitat_extent: LandscapeExtent,
}

impl Dispersal for InMemoryPrecalculatedDispersal {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        let location_index = (location.y() as usize) * (self.habitat_extent.width() as usize)
            + (location.x() as usize);

        let habitat_area =
            (self.habitat_extent.width() as usize) * (self.habitat_extent.height() as usize);

        let cumulative_dispersals_at_location = &self.cumulative_dispersal
            [location_index * habitat_area..(location_index + 1) * habitat_area];

        let cumulative_percentage_sample = rng.sample_uniform();

        let dispersal_target_index = match cumulative_dispersals_at_location.binary_search_by(|v| {
            v.partial_cmp(&cumulative_percentage_sample)
                .unwrap_or(Ordering::Equal)
        }) {
            Ok(index) | Err(index) => index,
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32,
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32,
        )
    }
}

impl InMemoryPrecalculatedDispersal {
    /// Creates a new `InMemoryPrecalculatedDispersal` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=WxH` where habitat has width `W`
    /// and height `W`.
    pub fn new(
        dispersal: &Array2D<f64>,
        habitat_extent: LandscapeExtent,
    ) -> Result<Self, InconsistentDispersalMapSize> {
        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if dispersal.num_rows() != habitat_area || dispersal.num_columns() != habitat_area {
            return Err(InconsistentDispersalMapSize);
        }

        let mut cumulative_dispersal = vec![0.0_f64; dispersal.num_elements()];

        for row_index in 0..dispersal.num_rows() {
            let sum: f64 = dispersal.row_iter(row_index).sum();
            let mut acc = 0.0_f64;

            for col_index in 0..dispersal.num_columns() {
                acc += dispersal[(row_index, col_index)];

                cumulative_dispersal[row_index * dispersal.row_len() + col_index] = acc / sum;
            }
        }

        Ok(Self {
            cumulative_dispersal,
            habitat_extent,
        })
    }
}

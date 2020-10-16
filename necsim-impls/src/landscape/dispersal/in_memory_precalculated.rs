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
    #[debug_requires(
        location.x() >= self.habitat_extent.x() &&
        location.x() < self.habitat_extent.x() + self.habitat_extent.width() &&
        location.y() >= self.habitat_extent.y() &&
        location.y() < self.habitat_extent.y() + self.habitat_extent.height()
    )]
    #[debug_ensures(
        ret.x() >= self.habitat_extent.x() &&
        ret.x() < self.habitat_extent.x() + self.habitat_extent.width() &&
        ret.y() >= self.habitat_extent.y() &&
        ret.y() < self.habitat_extent.y() + self.habitat_extent.height()
    )]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        let location_index = ((location.y() - self.habitat_extent.y()) as usize)
            * (self.habitat_extent.width() as usize)
            + ((location.x() - self.habitat_extent.x()) as usize);

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
            (dispersal_target_index % (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.x(),
            (dispersal_target_index / (self.habitat_extent.width() as usize)) as u32
                + self.habitat_extent.y(),
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
    #[debug_ensures(
        ret.is_ok() == (
            dispersal.num_columns() == old(
                (habitat_extent.width() * habitat_extent.height()) as usize
            ) && dispersal.num_rows() == old(
                (habitat_extent.width() * habitat_extent.height()) as usize
            )
        )
    )]
    // TODO: ensure correctness of cumulative_dispersal
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

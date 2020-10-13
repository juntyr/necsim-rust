use std::cmp::Ordering;

use array2d::Array2D;
use thiserror::Error;

use super::{Landscape, LandscapeExtent, Location};
use crate::rng;

pub struct LandscapeInMemoryWithPrecalculatedDispersal {
    habitat: Array2D<u32>,
    cumulative_dispersal: Vec<f64>,
}

#[derive(Error, Debug)]
#[error("The size of the dispersal map was inconsistent with the size of the habitat map.")]
pub struct InconsistentDispersalMapSize;

impl Landscape for LandscapeInMemoryWithPrecalculatedDispersal {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        #[allow(clippy::cast_possible_truncation)]
        LandscapeExtent::new(
            0,
            0,
            self.habitat.num_columns() as u32,
            self.habitat.num_rows() as u32,
        )
    }

    #[must_use]
    fn get_total_habitat(&self) -> u32 {
        self.habitat.elements_row_major_iter().sum()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat[(location.y() as usize, location.x() as usize)]
    }

    #[must_use]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        rng: &mut impl rng::Rng,
    ) -> Location {
        let location_index =
            (location.y() as usize) * self.habitat.row_len() + (location.x() as usize);

        let cumulative_dispersals_at_location = &self.cumulative_dispersal[location_index
            * self.habitat.num_elements()
            ..(location_index + 1) * self.habitat.num_elements()];

        let cumulative_percentage_sample = rng.sample_uniform();

        let dispersal_target_index = match cumulative_dispersals_at_location.binary_search_by(|v| {
            v.partial_cmp(&cumulative_percentage_sample)
                .unwrap_or(Ordering::Equal)
        }) {
            Ok(index) | Err(index) => index,
        };

        #[allow(clippy::cast_possible_truncation)]
        Location::new(
            (dispersal_target_index % self.habitat.row_len()) as u32,
            (dispersal_target_index / self.habitat.row_len()) as u32,
        )
    }
}

impl LandscapeInMemoryWithPrecalculatedDispersal {
    /// Creates a new `LandscapeInMemoryWithPrecalculatedDispersal` from the
    /// `habitat` and `dispersal` map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    pub fn new(
        habitat: Array2D<u32>,
        dispersal: &Array2D<f64>,
    ) -> Result<LandscapeInMemoryWithPrecalculatedDispersal, InconsistentDispersalMapSize> {
        if dispersal.num_rows() != habitat.num_elements()
            || dispersal.num_columns() != habitat.num_elements()
        {
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

        Ok(LandscapeInMemoryWithPrecalculatedDispersal {
            habitat,
            cumulative_dispersal,
        })
    }
}

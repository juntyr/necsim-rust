use std::cmp::Ordering;

use array2d::Array2D;
use thiserror::Error;

use necsim_core::landscape::{LandscapeExtent, Location};
use necsim_core::rng::Rng;

use super::Dispersal;
use crate::landscape::habitat::Habitat;

#[derive(Error, Debug)]
#[error("The size of the dispersal map was inconsistent with the size of the habitat map.")]
pub struct InconsistentDispersalMapSize;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryPrecalculatedDispersal {
    cumulative_dispersal: Vec<f64>,
    valid_dispersal_targets: Vec<Option<usize>>,
    habitat_extent: LandscapeExtent,
}

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

        let valid_dispersal_target_index = match self
            .valid_dispersal_targets
            .get(location_index * habitat_area + dispersal_target_index)
        {
            Some(Some(valid_dispersal_target_index)) => valid_dispersal_target_index,
            Some(None) => {
                println!("{:?}", cumulative_dispersals_at_location);
                panic!(
                    "sampled dispersal targets invalid target location {:?} {} {} {}",
                    location, location_index, dispersal_target_index, cumulative_percentage_sample
                )
            }
            None => panic!("sampled dispersal targets invalid target location v2"),
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
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            ) && dispersal.num_rows() == old(
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            )
        ),
        "returns error iff dispersal dimensions inconsistent"
    )]
    // TODO: We should ensure correctness of the cumulative_dispersal
    pub fn new(
        dispersal: &Array2D<f64>,
        habitat: &impl Habitat,
    ) -> Result<Self, InconsistentDispersalMapSize> {
        let habitat_extent = habitat.get_extent();

        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if dispersal.num_rows() != habitat_area || dispersal.num_columns() != habitat_area {
            return Err(InconsistentDispersalMapSize);
        }

        let mut cumulative_dispersal = vec![0.0_f64; dispersal.num_elements()];
        let mut valid_dispersal_targets = vec![None; dispersal.num_elements()];

        // !!!!!
        // TODO: Try multiplying all dispersal probs by the habitat they target
        // !!!!!

        for row_index in 0..dispersal.num_rows() {
            let sum: f64 = dispersal
                .row_iter(row_index)
                .enumerate()
                .map(|(col_index, dispersal_probability)| {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    dispersal_probability * f64::from(habitat.get_habitat_at_location(&location))
                })
                .sum();
            //let sum: f64 = dispersal.column_iter(row_index).sum();

            /*if sum < 0.0_f64 {
                panic!("{:?}", dispersal.row_iter(row_index).copied().collect::<Vec<f64>>());
            }

            let ox = row_index % habitat_extent.width() as usize;
            let oy = row_index / habitat_extent.width() as usize;

            println!("({},{}) => {}", ox, oy, sum);*/

            if sum > 0.0_f64 {
                let mut acc = 0.0_f64;
                let mut last_valid_target: Option<usize> = None;

                for col_index in 0..dispersal.num_columns() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    let dispersal_probability = dispersal[(row_index, col_index)]
                        * f64::from(habitat.get_habitat_at_location(&location));

                    //let dispersal_probability = dispersal[(col_index, row_index)];

                    if dispersal_probability > 0.0_f64 {
                        acc += dispersal_probability;

                        last_valid_target = Some(col_index);
                    }

                    cumulative_dispersal[row_index * dispersal.row_len() + col_index] = acc / sum;
                    valid_dispersal_targets[row_index * dispersal.row_len() + col_index] =
                        last_valid_target;
                }
            }
        }

        Ok(Self {
            cumulative_dispersal,
            valid_dispersal_targets,
            habitat_extent,
        })
    }
}

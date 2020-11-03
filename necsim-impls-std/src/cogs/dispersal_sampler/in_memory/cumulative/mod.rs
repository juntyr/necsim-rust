use array2d::{Array2D, Error};

use necsim_core::cogs::Habitat;
use necsim_core::landscape::{LandscapeExtent, Location};
use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

mod contract;
mod dispersal;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryCumulativeDispersalSampler {
    cumulative_dispersal: Vec<f64>,
    valid_dispersal_targets: Vec<Option<usize>>,
    habitat_extent: LandscapeExtent,
}

#[contract_trait]
impl<H: Habitat> InMemoryDispersalSampler<H> for InMemoryCumulativeDispersalSampler {
    /// Creates a new `InMemoryCumulativeDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(_)` is returned iff the dispersal `Array2D` cannot
    /// be constructed successfully.
    #[debug_ensures(ret.is_ok() -> ret.as_ref().unwrap()
        .explicit_only_valid_targets_dispersal_contract(old(habitat)),
        "valid_dispersal_targets only allows dispersal to habitat"
    )]
    //#[debug_ensures(..., "cumulative_dispersal stores the cumulative distribution function")]
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Result<Self, Error> {
        let habitat_extent = habitat.get_extent();

        let mut cumulative_dispersal = vec![0.0_f64; dispersal.num_elements()];
        let mut valid_dispersal_targets = vec![None; dispersal.num_elements()];

        for (row_index, row) in dispersal.rows_iter().enumerate() {
            let sum: f64 = row
                .enumerate()
                .map(|(col_index, dispersal_probability)| {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    dispersal_probability * f64::from(habitat.get_habitat_at_location(&location))
                })
                .sum();

            if sum > 0.0_f64 {
                let mut acc = 0.0_f64;
                let mut last_valid_target: Option<usize> = None;

                for col_index in 0..dispersal.num_columns() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let dispersal_probability = dispersal[(row_index, col_index)]
                        * f64::from(habitat.get_habitat_at_location(&location));

                    if dispersal_probability > 0.0_f64 {
                        acc += dispersal_probability;

                        last_valid_target = Some(col_index);
                    }

                    cumulative_dispersal[row_index * dispersal.row_len() + col_index] = acc / sum;

                    // Store the index of the last valid dispersal target
                    valid_dispersal_targets[row_index * dispersal.row_len() + col_index] =
                        last_valid_target;
                }
            }
        }

        Ok(InMemoryCumulativeDispersalSampler {
            cumulative_dispersal,
            valid_dispersal_targets,
            habitat_extent,
        })
    }
}

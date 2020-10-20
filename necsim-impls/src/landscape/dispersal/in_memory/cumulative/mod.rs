use array2d::Array2D;

use necsim_core::landscape::{LandscapeExtent, Location};

use crate::landscape::habitat::Habitat;

mod contract;
mod dispersal;

use crate::landscape::dispersal::in_memory::contract::explicit_in_memory_dispersal_check_contract;
use crate::landscape::dispersal::in_memory::error::InMemoryDispersalError;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryCumulativeDispersal {
    cumulative_dispersal: Vec<f64>,
    valid_dispersal_targets: Vec<Option<usize>>,
    habitat_extent: LandscapeExtent,
}

impl InMemoryCumulativeDispersal {
    /// Creates a new `InMemoryCumulativeDispersal` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=WxH` where habitat has width `W`
    /// and height `W`.
    ///
    /// `Err(InconsistentDispersalProbabilities)` is returned iff any of the
    /// following conditions is violated:
    /// - habitat cells must disperse somewhere
    /// - non-habitat cells must not disperse
    /// - dispersal must only target habitat cells
    #[debug_ensures(
        matches!(ret, Err(InMemoryDispersalError::InconsistentDispersalMapSize)) != (
            dispersal.num_columns() == old(
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            ) && dispersal.num_rows() == old(
                (habitat.get_extent().width() * habitat.get_extent().height()) as usize
            )
        ),
        "returns Err(InconsistentDispersalMapSize) iff dispersal dimensions inconsistent"
    )]
    #[debug_ensures(
        matches!(ret, Err(
            InMemoryDispersalError::InconsistentDispersalProbabilities
        )) != old(
            explicit_in_memory_dispersal_check_contract(dispersal, habitat)
        ), "returns Err(InconsistentDispersalMapSize) iff dispersal dimensions inconsistent"
    )]
    //#[debug_ensures(..., "cumulative_dispersal stores the cumulative distribution function")]
    #[debug_ensures(ret.is_ok() -> ret.as_ref().unwrap()
        .explicit_only_valid_targets_dispersal_contract(old(habitat)),
        "valid_dispersal_targets only allows dispersal to habitat"
    )]
    pub fn new(
        dispersal: &Array2D<f64>,
        habitat: &impl Habitat,
    ) -> Result<Self, InMemoryDispersalError> {
        let habitat_extent = habitat.get_extent();

        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if dispersal.num_rows() != habitat_area || dispersal.num_columns() != habitat_area {
            return Err(InMemoryDispersalError::InconsistentDispersalMapSize);
        }

        if !explicit_in_memory_dispersal_check_contract(dispersal, habitat) {
            return Err(InMemoryDispersalError::InconsistentDispersalProbabilities);
        }

        let mut cumulative_dispersal = vec![0.0_f64; dispersal.num_elements()];
        let mut valid_dispersal_targets = vec![None; dispersal.num_elements()];

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

        Ok(InMemoryCumulativeDispersal {
            cumulative_dispersal,
            valid_dispersal_targets,
            habitat_extent,
        })
    }
}

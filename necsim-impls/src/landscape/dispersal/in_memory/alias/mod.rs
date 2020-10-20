use array2d::Array2D;

use necsim_core::landscape::{LandscapeExtent, Location};

use crate::landscape::habitat::Habitat;

mod dispersal;

use crate::alias::AliasMethodSampler;
use crate::landscape::dispersal::in_memory::contract::explicit_in_memory_dispersal_check_contract;
use crate::landscape::dispersal::in_memory::error::InMemoryDispersalError;

#[allow(clippy::module_name_repetitions)]
pub struct InMemoryAliasDispersal {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    habitat_extent: LandscapeExtent,
}

impl InMemoryAliasDispersal {
    /// Creates a new `InMemoryAliasDispersal` from the
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

        let mut event_weights: Vec<(usize, f64)> = Vec::with_capacity(dispersal.row_len());

        let alias_dispersal = Array2D::from_iter_row_major(
            dispersal.rows_iter().map(|row| {
                event_weights.clear();

                for (col_index, dispersal_probability) in row.enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        (col_index % (habitat_extent.width() as usize)) as u32 + habitat_extent.x(),
                        (col_index / (habitat_extent.width() as usize)) as u32 + habitat_extent.y(),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let weight = dispersal_probability
                        * f64::from(habitat.get_habitat_at_location(&location));

                    if weight > 0.0_f64 {
                        event_weights.push((col_index, weight));
                    }
                }

                if event_weights.is_empty() {
                    None
                } else {
                    Some(AliasMethodSampler::new(&event_weights))
                }
            }),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        );

        Ok(Self {
            alias_dispersal,
            habitat_extent,
        })
    }
}

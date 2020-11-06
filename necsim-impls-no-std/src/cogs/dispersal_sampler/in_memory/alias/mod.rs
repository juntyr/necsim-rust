use alloc::vec::Vec;

use array2d::{Array2D, Error};

use necsim_core::cogs::Habitat;
use necsim_core::landscape::{LandscapeExtent, Location};

use crate::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

mod dispersal;

use crate::alias::AliasMethodSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemoryAliasDispersalSampler {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    habitat_extent: LandscapeExtent,
}

#[contract_trait]
impl<H: Habitat> InMemoryDispersalSampler<H> for InMemoryAliasDispersalSampler {
    /// Creates a new `InMemoryAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(_)` is returned iff the dispersal `Array2D` cannot
    /// be constructed successfully.
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Result<Self, Error> {
        let habitat_extent = habitat.get_extent();

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
        )?;

        Ok(Self {
            alias_dispersal,
            habitat_extent,
        })
    }
}

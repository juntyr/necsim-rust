use core::marker::PhantomData;

use alloc::vec::Vec;

use array2d::{Array2D, Error};

use necsim_core::cogs::{Habitat, RngCore};
use necsim_core::landscape::{LandscapeExtent, Location};

use crate::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

mod dispersal;

use crate::alias::AliasMethodSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemorySeparableAliasDispersalSampler<H: Habitat, G: RngCore> {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    self_dispersal: Array2D<f64>,
    habitat_extent: LandscapeExtent,
    _marker: PhantomData<(H, G)>,
}

#[contract_trait]
impl<H: Habitat, G: RngCore> InMemoryDispersalSampler<H, G>
    for InMemorySeparableAliasDispersalSampler<H, G>
{
    /// Creates a new `InMemorySeparableAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    ///
    /// # Errors
    ///
    /// `Err(_)` is returned iff the dispersal `Array2D` cannot
    /// be constructed successfully.
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Result<Self, Error> {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, f64)> = Vec::with_capacity(dispersal.row_len());

        let mut self_dispersal =
            Array2D::filled_with(0.0_f64, dispersal.num_rows(), dispersal.num_columns());

        let alias_dispersal = Array2D::from_iter_row_major(
            dispersal.rows_iter().enumerate().map(|(row_index, row)| {
                event_weights.clear();

                let mut self_dispersal_at_location = 0.0_f64;

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
                        // Separate self-dispersal from out-dispersal
                        if col_index == row_index {
                            self_dispersal_at_location = weight;
                        } else {
                            event_weights.push((col_index, weight));
                        }
                    }
                }

                let total_weight = event_weights.iter().map(|(_e, w)| *w).sum::<f64>()
                    + self_dispersal_at_location;

                if total_weight > 0.0_f64 {
                    self_dispersal[(
                        row_index / (habitat_extent.width() as usize),
                        row_index % (habitat_extent.width() as usize),
                    )] = self_dispersal_at_location / total_weight;
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
            self_dispersal,
            habitat_extent,
            _marker: PhantomData::<(H, G)>,
        })
    }
}

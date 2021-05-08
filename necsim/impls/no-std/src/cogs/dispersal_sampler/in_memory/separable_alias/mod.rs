use core::marker::PhantomData;

use alloc::vec::Vec;

use array2d::Array2D;

use necsim_core::{
    cogs::{Backup, Habitat, RngCore},
    landscape::Location,
};
use necsim_core_bond::ZeroInclOneInclF64;

use crate::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

mod dispersal;

use crate::alias::AliasMethodSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemorySeparableAliasDispersalSampler<H: Habitat, G: RngCore> {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    self_dispersal: Array2D<ZeroInclOneInclF64>,
    _marker: PhantomData<(H, G)>,
}

#[contract_trait]
impl<H: Habitat, G: RngCore> InMemoryDispersalSampler<H, G>
    for InMemorySeparableAliasDispersalSampler<H, G>
{
    /// Creates a new `InMemorySeparableAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Self {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, f64)> = Vec::with_capacity(dispersal.row_len());

        let mut self_dispersal = Array2D::filled_with(
            ZeroInclOneInclF64::zero(),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        );

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
                    // Safety: Normalisation limits the result to [0.0; 1.0]
                    let dispersal_probability = unsafe {
                        ZeroInclOneInclF64::new_unchecked(self_dispersal_at_location / total_weight)
                    };

                    self_dispersal[(
                        row_index / (habitat_extent.width() as usize),
                        row_index % (habitat_extent.width() as usize),
                    )] = dispersal_probability;
                }

                if event_weights.is_empty() {
                    None
                } else {
                    Some(AliasMethodSampler::new(&event_weights))
                }
            }),
            habitat_extent.height() as usize,
            habitat_extent.width() as usize,
        )
        .unwrap(); // infallible by PRE

        Self {
            alias_dispersal,
            self_dispersal,
            _marker: PhantomData::<(H, G)>,
        }
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore> Backup for InMemorySeparableAliasDispersalSampler<H, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal: self.alias_dispersal.clone(),
            self_dispersal: self.self_dispersal.clone(),
            _marker: PhantomData::<(H, G)>,
        }
    }
}

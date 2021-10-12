use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::{
    cogs::{Backup, Habitat, RngCore, F64Core},
    landscape::Location,
};

use crate::{
    alias::AliasMethodSampler, array2d::Array2D,
    cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler,
};

mod dispersal;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemoryAliasDispersalSampler<F: F64Core, H: Habitat<F>, G: RngCore<F>> {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    marker: PhantomData<(F, H, G)>,
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>, G: RngCore<F>> InMemoryDispersalSampler<F, H, G>
    for InMemoryAliasDispersalSampler<F, H, G>
{
    /// Creates a new `InMemoryAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<f64>, habitat: &H) -> Self {
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
        )
        .unwrap(); // infallible by PRE

        Self {
            alias_dispersal,
            marker: PhantomData::<(H, G)>,
        }
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>, G: RngCore<F>> Backup for InMemoryAliasDispersalSampler<F, H, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal: self.alias_dispersal.clone(),
            marker: PhantomData::<(H, G)>,
        }
    }
}

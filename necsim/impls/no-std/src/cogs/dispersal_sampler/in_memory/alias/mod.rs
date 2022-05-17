use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, Rng},
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

use crate::{
    alias::AliasMethodSampler, array2d::Array2D,
    cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler,
};

mod dispersal;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct InMemoryAliasDispersalSampler<M: MathsCore, H: Habitat<M>, G: Rng<M>> {
    alias_dispersal: Array2D<Option<AliasMethodSampler<usize>>>,
    marker: PhantomData<(M, H, G)>,
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> InMemoryDispersalSampler<M, H, G>
    for InMemoryAliasDispersalSampler<M, H, G>
{
    /// Creates a new `InMemoryAliasDispersalSampler` from the
    /// `dispersal` map and extent of the habitat map.
    fn unchecked_new(dispersal: &Array2D<NonNegativeF64>, habitat: &H) -> Self {
        let habitat_extent = habitat.get_extent();

        let mut event_weights: Vec<(usize, NonNegativeF64)> =
            Vec::with_capacity(dispersal.row_len());

        let alias_dispersal = Array2D::from_iter_row_major(
            dispersal.rows_iter().map(|row| {
                event_weights.clear();

                for (col_index, dispersal_probability) in row.enumerate() {
                    #[allow(clippy::cast_possible_truncation)]
                    let location = Location::new(
                        habitat_extent
                            .x()
                            .wrapping_add((col_index % usize::from(habitat_extent.width())) as u32),
                        habitat_extent
                            .y()
                            .wrapping_add((col_index / usize::from(habitat_extent.width())) as u32),
                    );

                    // Multiply all dispersal probabilities by the habitat of their target
                    let weight = *dispersal_probability
                        * NonNegativeF64::from(habitat.get_habitat_at_location(&location));

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
            usize::from(habitat_extent.height()),
            usize::from(habitat_extent.width()),
        )
        .unwrap(); // infallible by PRE

        Self {
            alias_dispersal,
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>, G: Rng<M>> Backup for InMemoryAliasDispersalSampler<M, H, G> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_dispersal: self.alias_dispersal.clone(),
            marker: PhantomData::<(M, H, G)>,
        }
    }
}

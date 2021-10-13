use alloc::vec::Vec;
use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::cogs::{Habitat, MathsCore};

use super::EqualDecomposition;

impl<M: MathsCore, H: Habitat<M>> EqualDecomposition<M, H> {
    /// # Errors
    ///
    /// Returns `Ok(Self)` iff the `habitat` can be partitioned into
    /// `partitions` by area, otherwise returns `Err(Self)`.
    pub fn area(habitat: &H, rank: u32, partitions: NonZeroU32) -> Result<Self, Self> {
        let extent = habitat.get_extent().clone();

        let mut indices = Vec::with_capacity(partitions.get() as usize);

        let morton_x = Self::next_log2(extent.width());
        let morton_y = Self::next_log2(extent.height());

        for location in extent.iter() {
            indices.push(Self::map_x_y_to_morton(
                morton_x,
                morton_y,
                location.x() - extent.x(),
                location.y() - extent.y(),
            ));
        }

        indices.sort_unstable();

        let num_indices = indices.len() as u64;
        let mut last_rank = 0;

        let indices: Vec<u64> = indices
            .into_iter()
            .enumerate()
            .filter_map(|(i, index)| {
                #[allow(clippy::cast_possible_truncation)]
                let next_rank = ((i as u64) * u64::from(partitions.get()) / num_indices) as u32;

                if next_rank == last_rank {
                    None
                } else {
                    last_rank = next_rank;

                    Some(index)
                }
            })
            .collect();

        let decomposition = Self {
            rank,
            partitions,

            extent,
            morton: (morton_x, morton_y),

            indices: indices.into_boxed_slice(),

            _marker: PhantomData::<(M, H)>,
        };

        if (decomposition.indices.len() + 1) == (partitions.get() as usize) {
            Ok(decomposition)
        } else {
            Err(decomposition)
        }
    }
}

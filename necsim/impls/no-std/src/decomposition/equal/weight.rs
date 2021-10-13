use alloc::vec::Vec;
use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::cogs::{F64Core, Habitat};

use super::EqualDecomposition;

impl<F: F64Core, H: Habitat<F>> EqualDecomposition<F, H> {
    /// # Errors
    ///
    /// Returns `Ok(Self)` iff the `habitat` can be partitioned into
    /// `partitions` by weight, otherwise returns `Err(Self)`.
    pub fn weight(habitat: &H, rank: u32, partitions: NonZeroU32) -> Result<Self, Self> {
        let extent = habitat.get_extent().clone();

        let mut total_habitat = 0;
        let mut indices = Vec::with_capacity(partitions.get() as usize);

        let morton_x = Self::next_log2(extent.width());
        let morton_y = Self::next_log2(extent.height());

        for location in extent.iter() {
            let h = habitat.get_habitat_at_location(&location);

            if h > 0 {
                total_habitat += u64::from(h);

                indices.push((
                    Self::map_x_y_to_morton(
                        morton_x,
                        morton_y,
                        location.x() - extent.x(),
                        location.y() - extent.y(),
                    ),
                    h,
                ));
            }
        }

        indices.sort_unstable();

        let mut cumulative_habitat = 0;
        let mut last_rank = 0;

        #[allow(clippy::type_complexity)]
        let indices: Vec<u64> = indices
            .into_iter()
            .filter_map(|(index, habitat)| {
                #[allow(clippy::cast_possible_truncation)]
                let next_rank = (u128::from(cumulative_habitat) * u128::from(partitions.get())
                    / u128::from(total_habitat)) as u32;

                cumulative_habitat += u64::from(habitat);

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

            _marker: PhantomData::<(F, H)>,
        };

        if (decomposition.indices.len() + 1) == (partitions.get() as usize) {
            Ok(decomposition)
        } else {
            Err(decomposition)
        }
    }
}

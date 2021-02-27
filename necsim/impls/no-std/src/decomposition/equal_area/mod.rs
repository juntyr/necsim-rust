use alloc::{boxed::Box, vec::Vec};
use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::{
    cogs::{Backup, Habitat},
    intrinsics::{ceil, log2},
    landscape::{LandscapeExtent, Location},
};

use crate::decomposition::Decomposition;

#[cfg(test)]
mod test;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct EqualAreaDecomposition<H: Habitat> {
    rank: u32,
    partitions: NonZeroU32,

    extent: LandscapeExtent,
    morton: (u8, u8),

    indices: Box<[u64]>,

    _marker: PhantomData<H>,
}

impl<H: Habitat> EqualAreaDecomposition<H> {
    /// # Errors
    ///
    /// Returns `Ok(Self)` iff the `habitat` can be partitioned into
    /// `partitions`, otherwise returns `Err(Self)`.
    pub fn new(habitat: &H, rank: u32, partitions: NonZeroU32) -> Result<Self, Self> {
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
                let next_rank = (i as u64) * u64::from(partitions.get()) / num_indices;

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

            _marker: PhantomData::<H>,
        };

        if u64::from(partitions.get())
            <= (u64::from(habitat.get_extent().width()) * u64::from(habitat.get_extent().height()))
        {
            Ok(decomposition)
        } else {
            Err(decomposition)
        }
    }

    fn next_log2(coord: u32) -> u8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if coord > 1 {
            ceil(log2(f64::from(coord))) as u8
        } else {
            0
        }
    }

    fn map_x_y_to_morton(mut morton_x: u8, mut morton_y: u8, mut dx: u32, mut dy: u32) -> u64 {
        let mut morton_index = 0_u64;

        let morton = morton_x.min(morton_y);

        for m in 0..morton {
            morton_index |= u64::from(dx & 0x1_u32) << (m * 2);
            dx >>= 1;

            morton_index |= u64::from(dy & 0x1_u32) << (m * 2 + 1);
            dy >>= 1;
        }

        morton_x -= morton;
        morton_y -= morton;

        if morton_x > 0 {
            morton_index |= u64::from(dx) << (morton * 2);
        }

        if morton_y > 0 {
            morton_index |= u64::from(dy) << (morton * 2 + morton_x);
        }

        morton_index
    }
}

#[contract_trait]
impl<H: Habitat> Backup for EqualAreaDecomposition<H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            rank: self.rank,
            partitions: self.partitions,
            extent: self.extent.clone(),
            morton: self.morton,
            indices: self.indices.clone(),
            _marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<H: Habitat> Decomposition<H> for EqualAreaDecomposition<H> {
    fn get_subdomain_rank(&self) -> u32 {
        self.rank
    }

    fn get_number_of_subdomains(&self) -> NonZeroU32 {
        self.partitions
    }

    #[debug_requires(
        habitat.get_extent() == &self.extent,
        "habitat has a matching extent"
    )]
    fn map_location_to_subdomain_rank(&self, location: &Location, habitat: &H) -> u32 {
        let mut dx = location.x() - self.extent.x();
        let mut dy = location.y() - self.extent.y();

        let morton_index = Self::map_x_y_to_morton(self.morton.0, self.morton.1, dx, dy);

        #[allow(clippy::cast_possible_truncation)]
        match self.indices.binary_search(&morton_index) {
            Ok(index) => (index + 1) as u32,
            Err(index) => index as u32,
        }
    }
}

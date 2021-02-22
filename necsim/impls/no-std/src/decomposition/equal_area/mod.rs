use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::{
    cogs::Habitat,
    intrinsics::{ceil, floor, log2},
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
    prev_log2: (u8, u8),

    offset: u32,
    offset_next_pow2: u32,

    _marker: PhantomData<H>,
}

impl<H: Habitat> EqualAreaDecomposition<H> {
    /// # Errors
    ///
    /// Returns `Ok(Self)` iff the `habitat` can be partitioned into
    /// `partitions`, otherwise returns `Err(Self)`.
    pub fn new(habitat: &H, rank: u32, partitions: NonZeroU32) -> Result<Self, Self> {
        let extent = habitat.get_extent().clone();

        let mut prev_log2_width = Self::prev_log2(extent.width());
        let mut prev_log2_height = Self::prev_log2(extent.height());

        let partitions_log2 = Self::next_log2(partitions.get());

        let successful_decomposition = partitions_log2 <= (prev_log2_width + prev_log2_height);

        // Favour partitions with similar width and height
        while partitions_log2 < (prev_log2_width + prev_log2_height) {
            #[allow(clippy::if_same_then_else)]
            if prev_log2_width == 0 {
                prev_log2_height -= 1;
            } else if prev_log2_height == 0 {
                prev_log2_width -= 1;
            } else if (extent.width() / (0x1_u32 << prev_log2_width))
                <= (extent.height() / (0x1_u32 << prev_log2_height))
            {
                prev_log2_width -= 1;
            } else {
                prev_log2_height -= 1;
            }
        }

        // Calculate the difference between partitions and 2^k
        let offset = partitions.get() - (0x1_u32 << Self::prev_log2(partitions.get()));
        let offset_next_pow2 = if offset > 0 {
            0x1_u32 << Self::next_log2(offset)
        } else {
            partitions.get()
        };

        let decomposition = Self {
            rank,
            partitions,

            extent,
            prev_log2: (prev_log2_width, prev_log2_height),

            offset,
            offset_next_pow2,

            _marker: PhantomData::<H>,
        };

        if successful_decomposition {
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

    fn prev_log2(coord: u32) -> u8 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        if coord > 1 {
            floor(log2(f64::from(coord))) as u8
        } else {
            0
        }
    }

    fn map_coord_to_index(coord: u32, length: u32, length_next_log2: u8) -> u32 {
        #[allow(clippy::cast_possible_truncation)]
        {
            (u64::from(coord) * (0x1_u64 << length_next_log2) / u64::from(length)) as u32
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
        let idx_x = Self::map_coord_to_index(
            location.x() - self.extent.x(),
            self.extent.width(),
            self.prev_log2.0,
        );
        let idx_y = Self::map_coord_to_index(
            location.y() - self.extent.y(),
            self.extent.height(),
            self.prev_log2.1,
        );

        let index = (idx_y << self.prev_log2.0) | idx_x;

        // Reduce the mapping from [0, 2^(k+1)) to [0, partitions)
        if index > self.offset_next_pow2 {
            (index >> 1) + self.offset
        } else {
            index
        }
    }
}

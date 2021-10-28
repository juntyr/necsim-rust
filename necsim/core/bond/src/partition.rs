use core::{convert::TryFrom, fmt, num::NonZeroU32};

use serde::Deserialize;

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct PartitionRankOutOfBounds(u32, u32);

impl fmt::Display for PartitionRankOutOfBounds {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "rank {} is is not in partition range [0, {}].",
            self.0, self.1
        )
    }
}

#[allow(clippy::module_name_repetitions, clippy::unsafe_derive_deserialize)]
#[derive(Copy, Clone, Debug, Deserialize, TypeLayout)]
#[serde(try_from = "PartitionRaw")]
pub struct Partition {
    rank: u32,
    size: NonZeroU32,
}

impl Partition {
    /// Creates a `Partition` from a `rank` and number of partitions.
    ///
    /// # Errors
    ///
    /// Returns `PartitionRankOutOfBounds` if `rank >= size`.
    pub const fn try_new(rank: u32, size: NonZeroU32) -> Result<Self, PartitionRankOutOfBounds> {
        if rank < size.get() {
            Ok(Self { rank, size })
        } else {
            Err(PartitionRankOutOfBounds(rank, size.get() - 1))
        }
    }

    /// Creates a `Partition` from a `rank` and number of partitions.
    ///
    /// # Safety
    ///
    /// The number of partitions must be strictly greater than `rank`.
    #[must_use]
    pub const unsafe fn new_unchecked(rank: u32, size: NonZeroU32) -> Self {
        Self { rank, size }
    }

    #[must_use]
    pub const fn monolithic() -> Self {
        Self {
            rank: 0,
            size: unsafe { NonZeroU32::new_unchecked(1) },
        }
    }

    #[must_use]
    pub const fn rank(self) -> u32 {
        self.rank
    }

    #[must_use]
    pub const fn size(self) -> NonZeroU32 {
        self.size
    }
}

impl TryFrom<PartitionRaw> for Partition {
    type Error = PartitionRankOutOfBounds;

    fn try_from(raw: PartitionRaw) -> Result<Self, Self::Error> {
        Self::try_new(raw.rank, raw.size)
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "Partition")]
struct PartitionRaw {
    rank: u32,
    size: NonZeroU32,
}

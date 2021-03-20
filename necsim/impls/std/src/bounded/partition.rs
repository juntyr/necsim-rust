use std::{convert::TryFrom, num::NonZeroU32};

use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Error)]
#[error("rank {0} is not in partition range [0, {1}].")]
#[allow(clippy::module_name_repetitions)]
pub struct PartitionRankOutOfBounds(u32, u32);

#[derive(Copy, Clone, Debug, Deserialize)]
#[serde(try_from = "PartitionRaw")]
#[allow(clippy::module_name_repetitions)]
pub struct Partition {
    rank: u32,
    partitions: NonZeroU32,
}

impl Partition {
    /// # Errors
    ///
    /// Returns `PartitionRankOutOfBounds` if `rank >= partitions`.
    pub fn try_new(rank: u32, partitions: NonZeroU32) -> Result<Self, PartitionRankOutOfBounds> {
        if rank < partitions.get() {
            Ok(Self { rank, partitions })
        } else {
            Err(PartitionRankOutOfBounds(rank, partitions.get() - 1))
        }
    }

    #[debug_ensures(
        ret < self.partitions().get(),
        "rank is in range [0, partitions)"
    )]
    #[must_use]
    pub fn rank(self) -> u32 {
        self.rank
    }

    #[must_use]
    pub fn partitions(self) -> NonZeroU32 {
        self.partitions
    }
}

impl TryFrom<PartitionRaw> for Partition {
    type Error = PartitionRankOutOfBounds;

    fn try_from(raw: PartitionRaw) -> Result<Self, Self::Error> {
        Self::try_new(raw.rank, raw.partitions)
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
struct PartitionRaw {
    rank: u32,
    partitions: NonZeroU32,
}

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
#[serde(deny_unknown_fields)]
#[serde(rename = "Partition")]
struct PartitionRaw {
    rank: u32,
    partitions: NonZeroU32,
}

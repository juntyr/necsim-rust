use std::num::{NonZeroU32, NonZeroU64, NonZeroUsize};

use serde::Deserialize;
use serde_state::DeserializeState;

use necsim_impls_no_std::bounded::{Partition, PositiveF64};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbsoluteDedupCache {
    pub capacity: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeDedupCache {
    pub factor: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithicParallelismMode {
    pub event_slice: NonZeroU32,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IsolatedParallelismMode {
    pub event_slice: NonZeroU32,
    pub partition: Partition,
}

#[derive(Debug, Deserialize)]
pub enum ParallelismMode {
    Monolithic(MonolithicParallelismMode),
    IsolatedIndividuals(IsolatedParallelismMode),
    IsolatedLandscape(IsolatedParallelismMode),
}

impl<'de> DeserializeState<'de, Partition> for ParallelismMode {
    fn deserialize_state<D>(partition: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;

        let parallelism_mode = ParallelismMode::deserialize(deserializer)?;

        match parallelism_mode {
            ParallelismMode::Monolithic(..)
            | ParallelismMode::IsolatedIndividuals(..)
            | ParallelismMode::IsolatedLandscape(..)
                if partition.partitions().get() > 1 =>
            {
                Err(D::Error::custom(format!(
                    "parallelism_mode {:?} is incompatible with non-monolithic partitioning.",
                    parallelism_mode
                )))
            },
            partition_mode => Ok(partition_mode),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CudaArguments {
    pub ptx_jit: bool,
    pub delta_t: PositiveF64,
    pub block_size: u32,
    pub grid_size: u32,
    pub step_slice: NonZeroU64,
    pub dedup_cache: DedupCache,
    pub parallelism_mode: ParallelismMode,
}

impl<'de> DeserializeState<'de, Partition> for CudaArguments {
    fn deserialize_state<D>(partition: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let raw = CudaArgumentsRaw::deserialize_state(partition, deserializer)?;

        let parallelism_mode = if let Some(parallelism_mode) = raw.parallelism_mode {
            parallelism_mode
        } else if partition.partitions().get() > 1 {
            return Err(serde::de::Error::custom(
                "The CUDA algorithm is (currently) incompatible with MPI partitioning.",
            ));
        } else {
            ParallelismMode::Monolithic(MonolithicParallelismMode {
                event_slice: NonZeroU32::new(1_000_000_u32).unwrap(),
            })
        };

        Ok(CudaArguments {
            ptx_jit: raw.ptx_jit,
            delta_t: raw.delta_t,
            block_size: raw.block_size,
            grid_size: raw.grid_size,
            step_slice: raw.step_slice,
            dedup_cache: raw.dedup_cache,
            parallelism_mode,
        })
    }
}

#[derive(Debug, DeserializeState)]
#[serde(default, deny_unknown_fields)]
#[serde(deserialize_state = "Partition")]
pub struct CudaArgumentsRaw {
    pub ptx_jit: bool,
    pub delta_t: PositiveF64,
    pub block_size: u32,
    pub grid_size: u32,
    pub step_slice: NonZeroU64,
    pub dedup_cache: DedupCache,
    #[serde(deserialize_state)]
    pub parallelism_mode: Option<ParallelismMode>,
}

impl Default for CudaArgumentsRaw {
    fn default() -> Self {
        Self {
            ptx_jit: false,
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            block_size: 32_u32,
            grid_size: 256_u32,
            step_slice: NonZeroU64::new(200_u64).unwrap(),
            dedup_cache: DedupCache::Relative(RelativeDedupCache {
                factor: PositiveF64::new(2.0_f64).unwrap(),
            }),
            parallelism_mode: None,
        }
    }
}

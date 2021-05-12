use std::num::{NonZeroU64, NonZeroUsize};

use serde::Deserialize;
use serde_state::DeserializeState;

use necsim_core_bond::{ClosedUnitF64, Partition, PositiveF64};

use necsim_impls_no_std::parallelisation::independent::{DedupCache, RelativeDedupCache};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MonolithicParallelismMode {
    pub event_slice: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IsolatedParallelismMode {
    pub event_slice: NonZeroUsize,
    pub partition: Partition,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProbabilisticParallelismMode {
    #[serde(alias = "communication")]
    pub communication_probability: ClosedUnitF64,
}

#[derive(Debug, Deserialize)]
pub enum ParallelismMode {
    Monolithic(MonolithicParallelismMode),
    IsolatedIndividuals(IsolatedParallelismMode),
    IsolatedLandscape(IsolatedParallelismMode),
    Individuals,
    Landscape,
    Probabilistic(ProbabilisticParallelismMode),
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
            ParallelismMode::Individuals
            | ParallelismMode::Landscape
            | ParallelismMode::Probabilistic(..)
                if partition.partitions().get() == 1 =>
            {
                Err(D::Error::custom(format!(
                    "parallelism_mode {:?} is incompatible with monolithic partitioning.",
                    parallelism_mode
                )))
            },
            partition_mode => Ok(partition_mode),
        }
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct IndependentArguments {
    pub delta_t: PositiveF64,
    pub step_slice: NonZeroU64,
    pub dedup_cache: DedupCache,
    pub parallelism_mode: ParallelismMode,
}

impl<'de> DeserializeState<'de, Partition> for IndependentArguments {
    fn deserialize_state<D>(partition: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let raw = IndependentArgumentsRaw::deserialize_state(partition, deserializer)?;

        let parallelism_mode = match raw.parallelism_mode {
            Some(parallelism_mode) => parallelism_mode,
            None => {
                if partition.partitions().get() > 1 {
                    ParallelismMode::Individuals
                } else {
                    ParallelismMode::Monolithic(MonolithicParallelismMode {
                        event_slice: NonZeroUsize::new(1_000_000_usize).unwrap(),
                    })
                }
            },
        };

        Ok(IndependentArguments {
            delta_t: raw.delta_t,
            step_slice: raw.step_slice,
            dedup_cache: raw.dedup_cache,
            parallelism_mode,
        })
    }
}

#[derive(Debug, DeserializeState)]
#[serde(default, deny_unknown_fields)]
#[serde(deserialize_state = "Partition")]
struct IndependentArgumentsRaw {
    delta_t: PositiveF64,
    step_slice: NonZeroU64,
    dedup_cache: DedupCache,
    #[serde(deserialize_state)]
    parallelism_mode: Option<ParallelismMode>,
}

impl Default for IndependentArgumentsRaw {
    fn default() -> Self {
        Self {
            delta_t: PositiveF64::new(1.0_f64).unwrap(),
            step_slice: NonZeroU64::new(10_u64).unwrap(),
            dedup_cache: DedupCache::Relative(RelativeDedupCache {
                factor: PositiveF64::new(2.0_f64).unwrap(),
            }),
            parallelism_mode: None,
        }
    }
}

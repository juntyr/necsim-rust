use serde::Deserialize;
use serde_state::DeserializeState;

use necsim_core_bond::{Partition, PositiveF64};

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct MonolithicArguments {
    pub parallelism_mode: ParallelismMode,
}

impl<'de> DeserializeState<'de, Partition> for MonolithicArguments {
    fn deserialize_state<D>(partition: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let raw = MonolithicArgumentsRaw::deserialize_state(partition, deserializer)?;

        let parallelism_mode = match raw.parallelism_mode {
            Some(parallelism_mode) => parallelism_mode,
            None => {
                if partition.partitions().get() > 1 {
                    ParallelismMode::OptimisticLockstep
                } else {
                    ParallelismMode::Monolithic
                }
            },
        };

        Ok(MonolithicArguments { parallelism_mode })
    }
}

#[derive(Default, Debug, DeserializeState)]
#[serde(default, deny_unknown_fields)]
#[serde(deserialize_state = "Partition")]
struct MonolithicArgumentsRaw {
    #[serde(deserialize_state)]
    parallelism_mode: Option<ParallelismMode>,
}

#[derive(Debug, Deserialize)]
pub struct OptimisticParallelismMode {
    pub delta_sync: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub struct AveragingParallelismMode {
    pub delta_sync: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum ParallelismMode {
    Monolithic,
    Optimistic(OptimisticParallelismMode),
    Lockstep,
    OptimisticLockstep,
    Averaging(AveragingParallelismMode),
}

impl<'de> DeserializeState<'de, Partition> for ParallelismMode {
    fn deserialize_state<D>(partition: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;

        let parallelism_mode = ParallelismMode::deserialize(deserializer)?;

        match parallelism_mode {
            ParallelismMode::Monolithic if partition.partitions().get() > 1 => {
                Err(D::Error::custom(format!(
                    "parallelism_mode {:?} is incompatible with non-monolithic partitioning.",
                    parallelism_mode
                )))
            },
            ParallelismMode::Optimistic(..)
            | ParallelismMode::Lockstep
            | ParallelismMode::OptimisticLockstep
            | ParallelismMode::Averaging(..)
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

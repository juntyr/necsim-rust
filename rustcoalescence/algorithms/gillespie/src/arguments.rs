use serde::{Deserialize, Serialize};
use serde_state::DeserializeState;

use necsim_core::reporter::Reporter;
use necsim_core_bond::PositiveF64;
use necsim_partitioning_core::{partition::Partition, LocalPartition};

#[derive(Serialize, Debug)]
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
                if partition.size().get() > 1 {
                    ParallelismMode::Lockstep
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

#[derive(Debug, Serialize, Deserialize)]
pub struct OptimisticParallelismMode {
    pub delta_sync: PositiveF64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AveragingParallelismMode {
    pub delta_sync: PositiveF64,
}

#[derive(Debug, Serialize, Deserialize)]
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
            ParallelismMode::Monolithic if partition.size().get() > 1 => {
                Err(D::Error::custom(format!(
                    "parallelism_mode {:?} is incompatible with non-monolithic partitioning.",
                    parallelism_mode
                )))
            },
            ParallelismMode::Optimistic(..)
            | ParallelismMode::Lockstep
            | ParallelismMode::OptimisticLockstep
            | ParallelismMode::Averaging(..)
                if partition.size().get() == 1 =>
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

#[must_use]
pub fn get_effective_monolithic_partition<'p, R: Reporter, P: LocalPartition<'p, R>>(
    args: &MonolithicArguments,
    local_partition: &P,
) -> Partition {
    match &args.parallelism_mode {
        ParallelismMode::Monolithic => Partition::monolithic(),
        ParallelismMode::Optimistic(_)
        | ParallelismMode::Lockstep
        | ParallelismMode::OptimisticLockstep
        | ParallelismMode::Averaging(_) => local_partition.get_partition(),
    }
}

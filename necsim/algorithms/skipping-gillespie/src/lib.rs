#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use serde::Deserialize;
use serde_state::DeserializeState;

use necsim_core::cogs::{
    CoherentLineageStore, Habitat, LineageReference, SeparableDispersalSampler,
};

use necsim_impls_no_std::{
    decomposition::Decomposition, partitioning::LocalPartition, reporter::ReporterContext,
};

use necsim_impls_std::{
    bounded::{Partition, PositiveF64},
    cogs::rng::pcg::Pcg,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;
mod simulate;

#[derive(Debug, Deserialize)]
pub struct OptimisticParallelismMode {
    delta_sync: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub struct AveragingParallelismMode {
    delta_sync: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum ParallelismMode {
    Monolithic,
    Optimistic(OptimisticParallelismMode),
    Lockstep,
    OptimisticLockstep,
    Averaging(AveragingParallelismMode),
}

#[derive(Debug)]
pub struct SkippingGillespieArguments {
    parallelism_mode: ParallelismMode,
}

impl<'de> DeserializeState<'de, Partition> for SkippingGillespieArguments {
    fn deserialize_state<D>(seed: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        use serde::de::Error;

        let raw = SkippingGillespieArgumentsRaw::deserialize(deserializer)?;

        let parallelism_mode = match raw.parallelism_mode {
            Some(parallelism_mode) => match parallelism_mode {
                ParallelismMode::Monolithic if seed.partitions().get() > 1 => {
                    return Err(D::Error::custom(format!(
                        "parallelism_mode {:?} is incompatible with non-monolithic partitioning.",
                        parallelism_mode
                    )))
                },
                ParallelismMode::Optimistic(..)
                | ParallelismMode::Lockstep
                | ParallelismMode::OptimisticLockstep
                | ParallelismMode::Averaging(..)
                    if seed.partitions().get() == 1 =>
                {
                    return Err(D::Error::custom(format!(
                        "parallelism_mode {:?} is incompatible with monolithic partitioning.",
                        parallelism_mode
                    )))
                },
                partition_mode => partition_mode,
            },
            None => {
                if seed.partitions().get() > 1 {
                    ParallelismMode::OptimisticLockstep
                } else {
                    ParallelismMode::Monolithic
                }
            },
        };

        Ok(SkippingGillespieArguments { parallelism_mode })
    }
}

#[derive(Debug, Deserialize)]
#[serde(default)]
struct SkippingGillespieArgumentsRaw {
    parallelism_mode: Option<ParallelismMode>,
}

impl Default for SkippingGillespieArgumentsRaw {
    fn default() -> Self {
        Self {
            parallelism_mode: None,
        }
    }
}

pub struct SkippingGillespieSimulation;

impl SkippingGillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on the `habitat` with `dispersal` and lineages from
    /// `lineage_store`.
    #[allow(clippy::too_many_arguments)]
    #[allow(clippy::needless_pass_by_value)]
    fn simulate<
        H: Habitat,
        D: SeparableDispersalSampler<H, Pcg>,
        R: LineageReference<H>,
        S: CoherentLineageStore<H, R>,
        P: ReporterContext,
        L: LocalPartition<P>,
        C: Decomposition<H>,
    >(
        habitat: H,
        dispersal_sampler: D,
        lineage_store: S,
        speciation_probability_per_generation: f64,
        seed: u64,
        local_partition: &mut L,
        decomposition: C,
        auxiliary: SkippingGillespieArguments,
    ) -> (f64, u64) {
        let (time, steps) = match auxiliary.parallelism_mode {
            ParallelismMode::Monolithic => simulate::monolithic::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
            ),
            ParallelismMode::Lockstep => simulate::lockstep::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
            ),
            ParallelismMode::Optimistic(OptimisticParallelismMode { delta_sync }) => {
                simulate::optimistic::simulate(
                    habitat,
                    dispersal_sampler,
                    lineage_store,
                    speciation_probability_per_generation,
                    seed,
                    local_partition,
                    decomposition,
                    delta_sync.get(),
                )
            },
            ParallelismMode::OptimisticLockstep => simulate::optimistic_lockstep::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
            ),
            ParallelismMode::Averaging(AveragingParallelismMode { delta_sync }) => {
                simulate::averaging::simulate(
                    habitat,
                    dispersal_sampler,
                    lineage_store,
                    speciation_probability_per_generation,
                    seed,
                    local_partition,
                    decomposition,
                    delta_sync.get(),
                )
            },
        };

        local_partition.report_progress_sync(0_u64);

        (time, steps)
    }
}

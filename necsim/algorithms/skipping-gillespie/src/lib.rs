#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use serde::Deserialize;

use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, SeparableDispersalSampler},
    reporter::Reporter,
};

use necsim_impls_no_std::{
    decomposition::Decomposition, partitioning::LocalPartition, reporter::ReporterContext,
};

use necsim_impls_std::{bounded::PositiveF64, cogs::rng::std::StdRng};

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
    Optimistic(OptimisticParallelismMode),
    Lockstep,
    OptimisticLockstep,
    Averaging(AveragingParallelismMode),
}

#[derive(Debug, Deserialize)]
#[serde(default)]
pub struct SkippingGillespieArguments {
    parallelism_mode: ParallelismMode,
}

impl Default for SkippingGillespieArguments {
    fn default() -> Self {
        Self {
            parallelism_mode: ParallelismMode::OptimisticLockstep,
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
        D: SeparableDispersalSampler<H, StdRng>,
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
        if local_partition.get_number_of_partitions().get() == 1 {
            log::warn!(
                "Parallelism mode {:?} is ignored in monolithic mode.",
                auxiliary.parallelism_mode
            );

            return simulate::monolithic::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
            );
        }

        let (time, steps) = match auxiliary.parallelism_mode {
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

        local_partition.get_reporter().report_progress(0_u64);

        (time, steps)
    }
}

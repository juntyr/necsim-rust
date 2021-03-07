#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use anyhow::Result;

use necsim_core::{
    cogs::{CoherentLineageStore, Habitat, LineageReference, SeparableDispersalSampler},
    reporter::Reporter,
};

use necsim_impls_no_std::{decomposition::Decomposition, partitioning::LocalPartition};

use necsim_impls_std::cogs::rng::std::StdRng;

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;
mod simulate;

#[derive(Copy, Clone, Debug)]
pub enum ParallelismMode {
    Optimistic(f64),
    Lockstep,
    OptimisticLockstep,
    Averaging(f64),
}

#[derive(Copy, Clone, Debug)]
pub struct SkippingGillespieArguments {
    pub parallelism_mode: ParallelismMode,
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
    ) -> Result<(f64, u64)> {
        if local_partition.get_number_of_partitions().get() == 1 {
            log::warn!(
                "Parallelism mode {:?} is ignored in monolithic mode.",
                auxiliary.parallelism_mode
            );

            return Ok(simulate::monolithic::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
            ));
        }

        anyhow::ensure!(
            match auxiliary.parallelism_mode {
                ParallelismMode::Optimistic(time_slice)
                | ParallelismMode::Averaging(time_slice) => time_slice > 0.0_f64,
                _ => true,
            },
            "Skipping-Gillespie algorithm parallelism_mode={:?} independent time step scalar must \
             be positive.",
            auxiliary.parallelism_mode,
        );

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
            ParallelismMode::Optimistic(independent_time_slice) => simulate::optimistic::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
                independent_time_slice,
            ),
            ParallelismMode::OptimisticLockstep => simulate::optimistic_lockstep::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
            ),
            ParallelismMode::Averaging(independent_time_slice) => simulate::averaging::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
                independent_time_slice,
            ),
        };

        local_partition.get_reporter().report_progress(0_u64);

        Ok((time, steps))
    }
}

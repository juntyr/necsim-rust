#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

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
    Optimistic,
    Lockstep,
    OptimisticLockstep,
}

#[derive(Copy, Clone, Debug)]
pub struct SkippingGillespieArguments {
    pub parallelism_mode: ParallelismMode,
}

impl Default for SkippingGillespieArguments {
    fn default() -> Self {
        Self {
            parallelism_mode: ParallelismMode::Optimistic,
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

        let partitioned_simulate = match auxiliary.parallelism_mode {
            ParallelismMode::Lockstep => simulate::lockstep::simulate,
            ParallelismMode::Optimistic => simulate::optimistic::simulate,
            ParallelismMode::OptimisticLockstep => simulate::optimistic_lockstep::simulate,
        };

        let (time, steps) = partitioned_simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
            decomposition,
        );

        local_partition.get_reporter().report_progress(0_u64);

        (time, steps)
    }
}

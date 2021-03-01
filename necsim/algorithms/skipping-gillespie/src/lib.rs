#![deny(clippy::pedantic)]
#![feature(never_type)]

#[macro_use]
extern crate contracts;

use necsim_core::cogs::{
    CoherentLineageStore, Habitat, LineageReference, SeparableDispersalSampler,
};

use necsim_impls_no_std::{decomposition::Decomposition, partitioning::LocalPartition};

use necsim_impls_std::cogs::rng::std::StdRng;

use necsim_impls_no_std::reporter::ReporterContext;

mod almost_infinite;
mod in_memory;
mod non_spatial;
mod simulate;

pub struct SkippingGillespieSimulation;

impl SkippingGillespieSimulation {
    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on the `habitat` with `dispersal` and lineages from
    /// `lineage_store`.
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
    ) -> (f64, u64) {
        if local_partition.get_number_of_partitions().get() == 1 {
            simulate::monolithic::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
            )
        } else {
            simulate::partitioned::simulate(
                habitat,
                dispersal_sampler,
                lineage_store,
                speciation_probability_per_generation,
                seed,
                local_partition,
                decomposition,
            )
        }
    }
}

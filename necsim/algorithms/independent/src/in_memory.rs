use array2d::Array2D;

use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{
    partitioning::{ParallelPartition, Partition, Partitioning},
    reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use super::IndependentSimulation;

#[contract_trait]
impl InMemorySimulation for IndependentSimulation {
    type AuxiliaryArguments = ();
    type Error = anyhow::Error;

    /// Simulates the independent coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        partitioning.with_local_partition(reporter_context, |partition| {
            let habitat = InMemoryHabitat::new(habitat.clone());
            let speciation_probability =
                UniformSpeciationProbability::new(speciation_probability_per_generation);
            let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;

            let lineage_origins = OriginPreSampler::all().percentage(sample_percentage);

            let lineages: Vec<Lineage> = InMemoryOriginSampler::new(
                match &partition {
                    Ok(_monolithic) => lineage_origins.partition(0, 1),
                    Err(parallel) => lineage_origins.partition(
                        parallel.get_partition_rank(),
                        parallel.get_number_of_partitions().get(),
                    ),
                },
                &habitat,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect();

            Ok(match partition {
                Ok(monolithic) => IndependentSimulation::simulate(
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineages,
                    seed,
                    monolithic.get_reporter(),
                ),
                Err(parallel) => {
                    let (partition_time, partition_steps) = IndependentSimulation::simulate(
                        habitat,
                        speciation_probability,
                        dispersal_sampler,
                        lineages,
                        seed,
                        parallel.get_reporter(),
                    );

                    parallel.reduce_global_time_steps(partition_time, partition_steps)
                },
            })
        })
    }
}

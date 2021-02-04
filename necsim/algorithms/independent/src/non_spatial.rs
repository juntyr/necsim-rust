use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    origin_sampler::{
        non_spatial::NonSpatialOriginSampler, percentage::PercentageOriginSampler,
        uniform_partition::UniformPartitionOriginSampler,
    },
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    partitioning::{ParallelPartition, Partition, Partitioning},
    reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::IndependentSimulation;

#[contract_trait]
impl NonSpatialSimulation for IndependentSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the independent coalescence algorithm on a non-spatial
    /// `habitat` with non-spatial `dispersal`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        partitioning.with_local_partition(reporter_context, |partition| {
            let habitat = NonSpatialHabitat::new(area, deme);
            let speciation_probability =
                UniformSpeciationProbability::new(speciation_probability_per_generation);
            let dispersal_sampler = NonSpatialDispersalSampler::default();

            let lineage_origins = PercentageOriginSampler::<NonSpatialHabitat>::new(
                NonSpatialOriginSampler::new(&habitat),
                sample_percentage,
            );
            let lineages = match &partition {
                Ok(_monolithic) => lineage_origins
                    .map(|indexed_location| Lineage::new(indexed_location, &habitat))
                    .collect(),
                Err(parallel) => UniformPartitionOriginSampler::new(
                    lineage_origins,
                    parallel.get_partition_rank() as usize,
                    parallel.get_number_of_partitions().get() as usize,
                )
                .map(|indexed_location| Lineage::new(indexed_location, &habitat))
                .collect(),
            };

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

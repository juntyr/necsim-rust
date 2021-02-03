use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    origin_sampler::{
        almost_infinite::AlmostInfiniteOriginSampler, percentage::PercentageOriginSampler,
        uniform_partition::UniformPartitionOriginSampler,
    },
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    partitioning::{ParallelPartition, Partition, Partitioning},
    reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::IndependentSimulation;

#[contract_trait]
impl AlmostInfiniteSimulation for IndependentSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the independent coalescence algorithm on an almost-infinite
    /// `habitat` with N(0, sigma) `dispersal`. Only a circular region with
    /// `radius` is sampled.
    fn simulate<P: Partitioning, R: ReporterContext>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        partitioning.with_local_partition(reporter_context, |partition| {
            let habitat = AlmostInfiniteHabitat::default();
            let speciation_probability =
                UniformSpeciationProbability::new(speciation_probability_per_generation);
            let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);

            let lineage_origins = PercentageOriginSampler::<AlmostInfiniteHabitat>::new(
                AlmostInfiniteOriginSampler::new(&habitat, radius),
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
                Err(parallel) => IndependentSimulation::simulate(
                    habitat,
                    speciation_probability,
                    dispersal_sampler,
                    lineages,
                    seed,
                    parallel.get_reporter(),
                ),
            })
        })
    }
}

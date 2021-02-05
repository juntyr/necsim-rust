use necsim_core::lineage::Lineage;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
    origin_sampler::{almost_infinite::AlmostInfiniteOriginSampler, pre_sampler::OriginPreSampler},
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

            let lineage_origins = OriginPreSampler::all().percentage(sample_percentage);

            let lineages = AlmostInfiniteOriginSampler::new(
                match &partition {
                    Ok(_monolithic) => lineage_origins.partition(0, 1),
                    Err(parallel) => lineage_origins.partition(
                        parallel.get_partition_rank(),
                        parallel.get_number_of_partitions().get(),
                    ),
                },
                &habitat,
                radius,
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

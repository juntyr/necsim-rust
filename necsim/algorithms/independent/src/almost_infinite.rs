use necsim_core::lineage::Lineage;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
        habitat::almost_infinite::AlmostInfiniteHabitat,
        origin_sampler::{
            almost_infinite::AlmostInfiniteOriginSampler,
            decomposition::DecompositionOriginSampler, pre_sampler::OriginPreSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    decomposition::radial::RadialDecomposition,
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::{IndependentArguments, IndependentSimulation, PartitionMode};

#[contract_trait]
impl AlmostInfiniteSimulation for IndependentSimulation {
    type AuxiliaryArguments = IndependentArguments;
    type Error = anyhow::Error;

    /// Simulates the independent coalescence algorithm on an almost-infinite
    /// `habitat` with N(0, sigma) `dispersal`. Only a circular region with
    /// `radius` is sampled.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma);

        let lineage_origins = OriginPreSampler::all().percentage(sample_percentage);
        let decomposition = RadialDecomposition::new(
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        );

        let lineages = match auxiliary.partition_mode {
            // Apply lineage origin partitioning in the `Individuals` mode
            PartitionMode::Individuals => AlmostInfiniteOriginSampler::new(
                lineage_origins.partition(
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions().get(),
                ),
                &habitat,
                radius,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect(),
            // Apply lineage origin decomposition in the `Landscape` mode
            PartitionMode::Landscape | PartitionMode::Probabilistic => {
                DecompositionOriginSampler::new(
                    AlmostInfiniteOriginSampler::new(lineage_origins, &habitat, radius),
                    &decomposition,
                )
                .map(|indexed_location| Lineage::new(indexed_location, &habitat))
                .collect()
            },
        };

        let (partition_time, partition_steps) = IndependentSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineages,
            seed,
            local_partition,
            decomposition,
            &auxiliary,
        )?;

        Ok(local_partition.reduce_global_time_steps(partition_time, partition_steps))
    }
}

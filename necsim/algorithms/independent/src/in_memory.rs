use array2d::Array2D;

use necsim_core::lineage::Lineage;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
        habitat::in_memory::InMemoryHabitat,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, in_memory::InMemoryOriginSampler,
            pre_sampler::OriginPreSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    decomposition::equal_area::EqualAreaDecomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use super::{IndependentArguments, IndependentSimulation, PartitionMode};

#[contract_trait]
impl InMemorySimulation for IndependentSimulation {
    type AuxiliaryArguments = IndependentArguments;
    type Error = anyhow::Error;

    /// Simulates the independent coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;

        let lineage_origins = OriginPreSampler::all().percentage(sample_percentage);
        let decomposition = match EqualAreaDecomposition::new(
            &habitat,
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        ) {
            Ok(decomposition) | Err(decomposition) => decomposition,
        };

        let lineages = match auxiliary.partition_mode {
            // Apply no lineage origin partitioning in the `Monolithic` mode
            PartitionMode::Monolithic => InMemoryOriginSampler::new(lineage_origins, &habitat)
                .map(|indexed_location| Lineage::new(indexed_location, &habitat))
                .collect(),
            // Apply lineage origin partitioning in the `Individuals` mode
            PartitionMode::Individuals => InMemoryOriginSampler::new(
                lineage_origins.partition(
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions().get(),
                ),
                &habitat,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect(),
            // Apply lineage origin partitioning in the `IsolatedIndividuals` mode
            PartitionMode::IsolatedIndividuals(partition) => InMemoryOriginSampler::new(
                lineage_origins.partition(partition.rank(), partition.partitions().get()),
                &habitat,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect(),
            // Apply lineage origin decomposition in the `Landscape` mode
            PartitionMode::Landscape | PartitionMode::Probabilistic => {
                DecompositionOriginSampler::new(
                    InMemoryOriginSampler::new(lineage_origins, &habitat),
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
            auxiliary,
        );

        Ok(local_partition.reduce_global_time_steps(partition_time, partition_steps))
    }
}

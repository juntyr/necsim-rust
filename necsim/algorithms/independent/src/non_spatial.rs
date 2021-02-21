use necsim_core::lineage::Lineage;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
        habitat::non_spatial::NonSpatialHabitat,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, non_spatial::NonSpatialOriginSampler,
            pre_sampler::OriginPreSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
    },
    decomposition::modulo::ModuloDecomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::{IndependentArguments, IndependentSimulation, PartitionMode};

#[contract_trait]
impl NonSpatialSimulation for IndependentSimulation {
    type AuxiliaryArguments = IndependentArguments;
    type Error = anyhow::Error;

    /// Simulates the independent coalescence algorithm on a non-spatial
    /// `habitat` with non-spatial `dispersal`.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineage_origins = OriginPreSampler::all().percentage(sample_percentage);
        let decomposition = ModuloDecomposition::new(
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        );

        let lineages = match auxiliary.partition_mode {
            // Apply lineage origin partitioning in the `Individuals` mode
            PartitionMode::Individuals => NonSpatialOriginSampler::new(
                lineage_origins.partition(
                    local_partition.get_partition_rank(),
                    local_partition.get_number_of_partitions().get(),
                ),
                &habitat,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect(),
            // Apply lineage origin decomposition in the `Landscape` mode
            PartitionMode::Landscape => DecompositionOriginSampler::new(
                NonSpatialOriginSampler::new(lineage_origins, &habitat),
                &decomposition,
            )
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect(),
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

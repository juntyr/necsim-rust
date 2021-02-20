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
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use super::{IndependentArguments, IndependentSimulation};

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

        let lineage_origins = OriginPreSampler::all()
            .percentage(sample_percentage)
            .partition(
                local_partition.get_partition_rank(),
                local_partition.get_number_of_partitions().get(),
            );

        let lineages: Vec<Lineage> = InMemoryOriginSampler::new(lineage_origins, &habitat)
            .map(|indexed_location| Lineage::new(indexed_location, &habitat))
            .collect();

        let (partition_time, partition_steps) = IndependentSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineages,
            seed,
            local_partition,
            &auxiliary,
        )?;

        Ok(local_partition.reduce_global_time_steps(partition_time, partition_steps))
    }
}

use array2d::Array2D;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
        habitat::in_memory::InMemoryHabitat,
        lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, in_memory::InMemoryOriginSampler,
            pre_sampler::OriginPreSampler,
        },
    },
    decomposition::equal_area::EqualAreaDecomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
    simulation::in_memory::InMemorySimulation,
};

use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use super::{SkippingGillespieArguments, SkippingGillespieSimulation};

#[contract_trait]
impl InMemorySimulation for SkippingGillespieSimulation {
    type AuxiliaryArguments = SkippingGillespieArguments;
    type Error = anyhow::Error;

    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on an in-memory `habitat` with precalculated `dispersal`.
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
        let dispersal_sampler = InMemorySeparableAliasDispersalSampler::new(dispersal, &habitat)?;

        let decomposition = match EqualAreaDecomposition::new(
            &habitat,
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        ) {
            Ok(decomposition) | Err(decomposition) => decomposition,
        };

        let lineage_store = if local_partition.get_number_of_partitions().get() > 1 {
            CoherentInMemoryLineageStore::new(DecompositionOriginSampler::new(
                InMemoryOriginSampler::new(
                    OriginPreSampler::all().percentage(sample_percentage),
                    &habitat,
                ),
                &decomposition,
            ))
        } else {
            CoherentInMemoryLineageStore::new(InMemoryOriginSampler::new(
                OriginPreSampler::all().percentage(sample_percentage),
                &habitat,
            ))
        };

        SkippingGillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
            decomposition,
            auxiliary,
        )
    }
}

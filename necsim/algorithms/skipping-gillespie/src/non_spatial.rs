use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
        habitat::non_spatial::NonSpatialHabitat,
        lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
        origin_sampler::{
            decomposition::DecompositionOriginSampler, non_spatial::NonSpatialOriginSampler,
            pre_sampler::OriginPreSampler,
        },
    },
    decomposition::modulo::ModuloDecomposition,
    partitioning::LocalPartition,
    reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::SkippingGillespieSimulation;

#[contract_trait]
impl NonSpatialSimulation for SkippingGillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm with self-dispersal event
    /// skipping on a non-spatial `habitat` with non-spatial `dispersal`.
    fn simulate<R: ReporterContext, P: LocalPartition<R>>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        local_partition: &mut P,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let decomposition = ModuloDecomposition::new(
            local_partition.get_partition_rank(),
            local_partition.get_number_of_partitions(),
        );

        let lineage_store = if local_partition.get_number_of_partitions().get() > 1 {
            CoherentInMemoryLineageStore::new(DecompositionOriginSampler::new(
                NonSpatialOriginSampler::new(
                    OriginPreSampler::all().percentage(sample_percentage),
                    &habitat,
                ),
                &decomposition,
            ))
        } else {
            CoherentInMemoryLineageStore::new(NonSpatialOriginSampler::new(
                OriginPreSampler::all().percentage(sample_percentage),
                &habitat,
            ))
        };

        Ok(SkippingGillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
            decomposition,
        ))
    }
}

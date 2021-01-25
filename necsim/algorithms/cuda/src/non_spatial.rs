use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::non_spatial::NonSpatialSimulation,
};

use super::CudaSimulation;

#[contract_trait]
impl NonSpatialSimulation for CudaSimulation {
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on a
    /// non-spatial `habitat` with non-spatial `dispersal`.
    fn simulate<P: ReporterContext>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();
        let lineage_store = IncoherentInMemoryLineageStore::new(sample_percentage, &habitat);

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            reporter_context,
        )
    }
}

use necsim_core::cogs::LineageStore;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::non_spatial::NonSpatialSimulation,
};

use super::{CudaArguments, CudaSimulation};

#[contract_trait]
impl NonSpatialSimulation for CudaSimulation {
    type AuxiliaryArguments = CudaArguments;
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
        auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineages =
            CoherentInMemoryLineageStore::new(sample_percentage, &habitat).into_lineages();

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineages,
            speciation_probability_per_generation,
            seed,
            reporter_context,
            &auxiliary,
        )
    }
}

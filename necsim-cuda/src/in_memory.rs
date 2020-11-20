use array2d::Array2D;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{reporter::ReporterContext, simulation::in_memory::InMemorySimulation};

use super::CudaSimulation;

#[contract_trait]
impl InMemorySimulation for CudaSimulation {
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an in
    /// memory `habitat` with precalculated `dispersal`.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    fn simulate<P: ReporterContext>(
        habitat: &Array2D<u32>,
        dispersal: &Array2D<f64>,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryPackedAliasDispersalSampler::new(dispersal, &habitat)?;

        CudaSimulation::simulate(
            habitat,
            dispersal_sampler,
            speciation_probability_per_generation,
            sample_percentage,
            seed,
            reporter_context,
        )
    }
}

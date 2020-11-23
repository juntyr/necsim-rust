use necsim_impls_no_std::cogs::{
    dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
    habitat::almost_infinite::AlmostInfiniteHabitat,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::almost_infinite::AlmostInfiniteSimulation,
};

use super::CudaSimulation;

#[contract_trait]
impl AlmostInfiniteSimulation for CudaSimulation {
    type Error = anyhow::Error;

    /// Simulates the coalescence algorithm on a CUDA-capable GPU on an
    /// almost-infinite `habitat` with N(0, sigma) `dispersal`. Only a
    /// circular region with `radius` is sampled.
    fn simulate<P: ReporterContext>(
        radius: u32,
        sigma: f64,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(sigma, &habitat);

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

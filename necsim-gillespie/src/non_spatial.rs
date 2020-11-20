use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::non_spatial::NonSpatialSimulation,
};

use super::GillespieSimulation;

#[contract_trait]
impl NonSpatialSimulation for GillespieSimulation {
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm on a non-spatial
    /// `habitat` with non-spatial `dispersal`.
    fn simulate<P: ReporterContext>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::new(&habitat);

        Ok(GillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            speciation_probability_per_generation,
            sample_percentage,
            seed,
            reporter_context,
        ))
    }
}

use necsim_core::cogs::LineageStore;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::non_spatial::NonSpatialSimulation,
};

use super::IndependentSimulation;

#[contract_trait]
impl NonSpatialSimulation for IndependentSimulation {
    type Error = !;

    /// Simulates the independent coalescence algorithm on a non-spatial
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
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineages =
            CoherentInMemoryLineageStore::new(sample_percentage, &habitat).into_lineages();

        Ok(IndependentSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineages,
            seed,
            reporter_context,
        ))
    }
}

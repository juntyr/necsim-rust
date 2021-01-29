use array2d::Array2D;

use necsim_core::cogs::LineageStore;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    speciation_probability::uniform::UniformSpeciationProbability,
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{reporter::ReporterContext, simulation::in_memory::InMemorySimulation};

use super::IndependentSimulation;

#[contract_trait]
impl InMemorySimulation for IndependentSimulation {
    type Error = anyhow::Error;

    /// Simulates the independent coalescence algorithm on an in memory
    /// `habitat` with precalculated `dispersal`.
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
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;

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
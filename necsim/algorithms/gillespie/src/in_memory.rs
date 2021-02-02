use array2d::Array2D;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    habitat::in_memory::InMemoryHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    origin_sampler::{in_memory::InMemoryOriginSampler, percentage::PercentageOriginSampler},
};
use necsim_impls_std::cogs::dispersal_sampler::in_memory::InMemoryDispersalSampler;

use necsim_impls_no_std::{reporter::ReporterContext, simulation::in_memory::InMemorySimulation};

use super::GillespieSimulation;

#[contract_trait]
impl InMemorySimulation for GillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = anyhow::Error;

    /// Simulates the Gillespie coalescence algorithm on an in memory
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
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = InMemoryHabitat::new(habitat.clone());
        let dispersal_sampler = InMemoryAliasDispersalSampler::new(dispersal, &habitat)?;
        let lineage_store = CoherentInMemoryLineageStore::new(PercentageOriginSampler::new(
            InMemoryOriginSampler::new(&habitat),
            sample_percentage,
        ));

        Ok(GillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            reporter_context,
        ))
    }
}

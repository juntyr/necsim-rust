use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, percentage::PercentageOriginSampler},
};

use necsim_impls_no_std::{
    partitioning::Partitioning, reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::GillespieSimulation;

#[contract_trait]
impl NonSpatialSimulation for GillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm on a non-spatial
    /// `habitat` with non-spatial `dispersal`.
    fn simulate<P: Partitioning, R: ReporterContext>(
        area: (u32, u32),
        deme: u32,
        speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        _partitioning: &mut P,
        reporter_context: R,
        _auxiliary: Self::AuxiliaryArguments,
    ) -> Result<(f64, u64), Self::Error> {
        let habitat = NonSpatialHabitat::new(area, deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();
        let lineage_store = CoherentInMemoryLineageStore::new(PercentageOriginSampler::new(
            NonSpatialOriginSampler::new(&habitat),
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

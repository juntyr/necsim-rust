use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::globally::gillespie::GillespieLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::GillespieSimulation;

#[contract_trait]
impl NonSpatialSimulation for GillespieSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the Gillespie coalescence algorithm on a non-spatial
    /// `habitat` with non-spatial `dispersal`.
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

        let lineage_store = GillespieLineageStore::new(NonSpatialOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
        ));

        Ok(GillespieSimulation::simulate(
            habitat,
            dispersal_sampler,
            lineage_store,
            speciation_probability_per_generation,
            seed,
            local_partition,
        ))
    }
}

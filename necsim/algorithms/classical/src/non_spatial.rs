use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::locally::classical::ClassicalLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    partitioning::LocalPartition, reporter::ReporterContext,
    simulation::non_spatial::NonSpatialSimulation,
};

use super::ClassicalSimulation;

#[contract_trait]
impl NonSpatialSimulation for ClassicalSimulation {
    type AuxiliaryArguments = ();
    type Error = !;

    /// Simulates the classical coalescence algorithm on a non-spatial
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
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation);
        let dispersal_sampler = NonSpatialDispersalSampler::default();

        let lineage_store = ClassicalLineageStore::new(NonSpatialOriginSampler::new(
            OriginPreSampler::all().percentage(sample_percentage),
            &habitat,
        ));

        Ok(ClassicalSimulation::simulate(
            habitat,
            speciation_probability,
            dispersal_sampler,
            lineage_store,
            seed,
            local_partition,
        ))
    }
}

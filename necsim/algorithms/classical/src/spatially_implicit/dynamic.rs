use necsim_impls_no_std::cogs::{
    dispersal_sampler::spatially_implicit::SpatiallyImplicitDispersalSampler,
    habitat::spatially_implicit::SpatiallyImplicitHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    origin_sampler::{
        pre_sampler::OriginPreSampler, spatially_implicit::SpatiallyImplicitOriginSampler,
    },
    speciation_probability::spatially_implicit::SpatiallyImplicitSpeciationProbability,
};

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

use super::ClassicalSimulation;

/// Simulates the classical coalescence algorithm on non-spatial
/// local and meta `habitat`s with non-spatial `dispersal` and
/// migration from the meta- to the local community.
/// The metacommunity is assumed to be dynamic.
#[allow(clippy::too_many_arguments, clippy::module_name_repetitions)]
pub fn simulate_dynamic<R: ReporterContext, P: LocalPartition<R>>(
    local_area_deme: ((u32, u32), u32),
    meta_area_deme: ((u32, u32), u32),
    local_migration_probability_per_generation: f64,
    meta_speciation_probability_per_generation: f64,
    sample_percentage: f64,
    seed: u64,
    local_partition: &mut P,
) -> (f64, u64) {
    let habitat = SpatiallyImplicitHabitat::new(
        local_area_deme.0,
        local_area_deme.1,
        meta_area_deme.0,
        meta_area_deme.1,
    );
    let speciation_probability =
        SpatiallyImplicitSpeciationProbability::new(meta_speciation_probability_per_generation);
    let dispersal_sampler =
        SpatiallyImplicitDispersalSampler::new(local_migration_probability_per_generation);

    let lineage_store = CoherentInMemoryLineageStore::new(SpatiallyImplicitOriginSampler::new(
        OriginPreSampler::all().percentage(sample_percentage),
        &habitat,
    ));

    ClassicalSimulation::simulate(
        habitat,
        speciation_probability,
        dispersal_sampler,
        lineage_store,
        seed,
        local_partition,
    )
}

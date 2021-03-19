use std::collections::HashSet;

use necsim_core::cogs::{Habitat, RngCore, RngSampler};

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    partitioning::{monolithic::MonolithicLocalPartition, LocalPartition},
    reporter::{GuardedReporter, ReporterContext},
};

use necsim_impls_std::{cogs::rng::std::StdRng, reporter::biodiversity::BiodiversityReporter};

use super::ClassicalSimulation;

/// Simulates the classical coalescence algorithm on non-spatial
/// local and meta `habitat`s with non-spatial `dispersal` and
/// migration from the meta- to the local community.
/// The metacommunity is assumed to be static.
#[allow(clippy::too_many_arguments, clippy::module_name_repetitions)]
pub fn simulate_static<R: ReporterContext, P: LocalPartition<R>>(
    local_area_deme: ((u32, u32), u32),
    meta_area_deme: ((u32, u32), u32),
    local_migration_probability_per_generation: f64,
    meta_speciation_probability_per_generation: f64,
    sample_percentage: f64,
    seed: u64,
    local_partition: &mut P,
) -> (f64, u64) {
    let local_habitat = NonSpatialHabitat::new(local_area_deme.0, local_area_deme.1);
    let local_speciation_probability =
        UniformSpeciationProbability::new(local_migration_probability_per_generation);
    let local_dispersal_sampler = NonSpatialDispersalSampler::default();

    let local_lineage_store = CoherentInMemoryLineageStore::new(NonSpatialOriginSampler::new(
        OriginPreSampler::all().percentage(sample_percentage),
        &local_habitat,
    ));

    let mut number_of_migrations = 0_usize;

    // TODO: How can we still report the local events (without risking of confusion)
    // TODO: Two stage migration should also be distributed
    let (local_time, local_steps) = ClassicalSimulation::simulate::<
        NonSpatialHabitat,
        UniformSpeciationProbability,
        NonSpatialDispersalSampler<_>,
        InMemoryLineageReference,
        CoherentInMemoryLineageStore<_>,
        MigrationReporterContext<_>,
        MonolithicLocalPartition<_>,
    >(
        local_habitat,
        local_speciation_probability,
        local_dispersal_sampler,
        local_lineage_store,
        seed,
        &mut MonolithicLocalPartition::from_reporter(
            MigrationReporterContext::new(|migration_reporter| {
                number_of_migrations = migration_reporter.biodiversity()
            })
            .build_guarded(),
        ),
    );

    let meta_habitat = NonSpatialHabitat::new(meta_area_deme.0, meta_area_deme.1);

    // TODO: get 'next' seed from first simulation somehow
    // IDEALLY: the rng would be accessible here somehow so at least the type is the
    // same
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0x8000_0000_0000_0000_u64));

    let mut unique_migration_targets = HashSet::with_capacity(number_of_migrations);

    #[allow(clippy::cast_possible_truncation)]
    let max_unique_location_index = meta_habitat.get_total_habitat() as usize;

    for _ in 0..number_of_migrations {
        unique_migration_targets.insert(rng.sample_index(max_unique_location_index));
    }

    let meta_speciation_probability =
        UniformSpeciationProbability::new(meta_speciation_probability_per_generation);
    let meta_dispersal_sampler = NonSpatialDispersalSampler::default();

    #[allow(clippy::cast_precision_loss)]
    let meta_lineage_store = CoherentInMemoryLineageStore::new(NonSpatialOriginSampler::new(
        OriginPreSampler::all().percentage(
            (unique_migration_targets.len() as f64) / (max_unique_location_index as f64),
        ),
        &meta_habitat,
    ));

    let (meta_time, meta_steps) = ClassicalSimulation::simulate(
        meta_habitat,
        meta_speciation_probability,
        meta_dispersal_sampler,
        meta_lineage_store,
        rng.sample_u64(),
        local_partition,
    );

    (local_time + meta_time, local_steps + meta_steps)
}

struct MigrationReporterContext<F: FnOnce(BiodiversityReporter)> {
    finaliser: F,
}

impl<F: FnOnce(BiodiversityReporter)> MigrationReporterContext<F> {
    pub fn new(finaliser: F) -> Self {
        Self { finaliser }
    }
}

impl<F: FnOnce(BiodiversityReporter)> ReporterContext for MigrationReporterContext<F> {
    type Finaliser = F;
    type Reporter = BiodiversityReporter;

    fn build_guarded(self) -> GuardedReporter<Self::Reporter, Self::Finaliser> {
        GuardedReporter::from(BiodiversityReporter::default(), self.finaliser)
    }
}

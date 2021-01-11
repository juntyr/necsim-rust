use std::collections::HashSet;

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_store::coherent::in_memory::CoherentInMemoryLineageStore,
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    reporter::ReporterContext, simulation::non_spatial_migration::NonSpatialMigrationSimulation,
};

use necsim_impls_std::{cogs::rng::std::StdRng, reporter::biodiversity::BiodiversityReporter};

use necsim_core::cogs::{Habitat, RngCore, RngSampler};

use super::ClassicalSimulation;

#[contract_trait]
impl NonSpatialMigrationSimulation for ClassicalSimulation {
    type Error = !;

    /// Simulates the classical coalescence algorithm on non-spatial
    /// local and meta `habitat`s with non-spatial `dispersal` and
    /// migration from the meta- to the local community.
    #[allow(clippy::too_many_arguments)]
    fn simulate<P: ReporterContext>(
        local_area_deme: ((u32, u32), u32),
        meta_area_deme: ((u32, u32), u32),
        local_migration_probability_per_generation: f64,
        meta_speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error> {
        let local_habitat = NonSpatialHabitat::new(local_area_deme.0, local_area_deme.1);
        let local_speciation_probability =
            UniformSpeciationProbability::new(local_migration_probability_per_generation);
        let local_dispersal_sampler = NonSpatialDispersalSampler::new(&local_habitat);
        let local_lineage_store =
            CoherentInMemoryLineageStore::new(sample_percentage, &local_habitat);

        let mut migration_reporter = BiodiversityReporter::default();

        // TODO: Can we really discard all events during the local simulation?
        let (local_time, local_steps) = ClassicalSimulation::simulate(
            local_habitat,
            local_speciation_probability,
            local_dispersal_sampler,
            local_lineage_store,
            seed,
            MigrationReporterContext::new(&mut migration_reporter),
        );

        let number_of_migrations = migration_reporter.biodiversity();

        let meta_habitat = NonSpatialHabitat::new(meta_area_deme.0, meta_area_deme.1);

        // TODO: get 'next' seed from first simulation somehow
        // IDEALLY: the rng would be accessible here somehow so at least the type is the
        //          same
        let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0x8000_0000_0000_0000_u64));

        let mut unique_migration_targets = HashSet::with_capacity(number_of_migrations);

        #[allow(clippy::cast_possible_truncation)]
        let max_unique_location_index = meta_habitat.get_total_habitat() as usize;

        for _ in 0..number_of_migrations {
            unique_migration_targets.insert(rng.sample_index(max_unique_location_index));
        }

        let meta_speciation_probability =
            UniformSpeciationProbability::new(meta_speciation_probability_per_generation);
        let meta_dispersal_sampler = NonSpatialDispersalSampler::new(&meta_habitat);
        #[allow(clippy::cast_precision_loss)]
        let meta_lineage_store = CoherentInMemoryLineageStore::new(
            (unique_migration_targets.len() as f64) / (max_unique_location_index as f64),
            &meta_habitat,
        );

        let (meta_time, meta_steps) = ClassicalSimulation::simulate(
            meta_habitat,
            meta_speciation_probability,
            meta_dispersal_sampler,
            meta_lineage_store,
            rng.sample_u64(),
            reporter_context,
        );

        Ok((local_time + meta_time, local_steps + meta_steps))
    }
}

struct MigrationReporterContext<'r> {
    migration_reporter: &'r mut BiodiversityReporter,
}

impl<'r> MigrationReporterContext<'r> {
    pub fn new(migration_reporter: &'r mut BiodiversityReporter) -> Self {
        Self { migration_reporter }
    }
}

impl<'r> ReporterContext for MigrationReporterContext<'r> {
    type Reporter = BiodiversityReporter;

    fn with_reporter<O, F: FnOnce(&mut Self::Reporter) -> O>(self, inner: F) -> O {
        inner(self.migration_reporter)
    }
}

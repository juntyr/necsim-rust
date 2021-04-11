use std::collections::HashSet;

use necsim_core::{
    cogs::{Habitat, RngCore, RngSampler},
    event::SpeciationEvent,
    impl_report,
    reporter::{boolean::Boolean, FilteredReporter, Reporter},
};

use necsim_impls_no_std::cogs::{
    dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
    habitat::non_spatial::NonSpatialHabitat,
    lineage_reference::in_memory::InMemoryLineageReference,
    lineage_store::coherent::locally::classical::ClassicalLineageStore,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
    speciation_probability::uniform::UniformSpeciationProbability,
};

use necsim_impls_no_std::{
    partitioning::{monolithic::live::LiveMonolithicLocalPartition, LocalPartition},
    reporter::ReporterContext,
};

use necsim_impls_std::cogs::rng::std::StdRng;

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
    if u64::from(meta_area_deme.0 .0) * u64::from(meta_area_deme.0 .1) * u64::from(meta_area_deme.1)
        == 0
    {
        return (0.0_f64, 0_u64);
    }

    let local_habitat = NonSpatialHabitat::new(local_area_deme.0, local_area_deme.1);
    let local_speciation_probability =
        UniformSpeciationProbability::new(local_migration_probability_per_generation);
    let local_dispersal_sampler = NonSpatialDispersalSampler::default();

    let local_lineage_store = ClassicalLineageStore::new(NonSpatialOriginSampler::new(
        OriginPreSampler::all().percentage(sample_percentage),
        &local_habitat,
    ));

    let mut number_of_migrations = 0_usize;

    let rng = StdRng::seed_from_u64(seed);

    // TODO: How can we still report the local events (without risking of confusion)
    // TODO: Two stage migration should also be distributed
    let (local_time, local_steps, mut rng) = ClassicalSimulation::simulate_chain::<
        NonSpatialHabitat,
        StdRng,
        UniformSpeciationProbability,
        NonSpatialDispersalSampler<_>,
        InMemoryLineageReference,
        ClassicalLineageStore<_>,
        MigrationReporter,
        LiveMonolithicLocalPartition<_>,
    >(
        local_habitat,
        local_speciation_probability,
        local_dispersal_sampler,
        local_lineage_store,
        rng,
        &mut LiveMonolithicLocalPartition::from_reporter(FilteredReporter::from(
            MigrationReporter::new(&mut number_of_migrations),
        )),
    );

    let meta_habitat = NonSpatialHabitat::new(meta_area_deme.0, meta_area_deme.1);

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
    let meta_lineage_store = ClassicalLineageStore::new(NonSpatialOriginSampler::new(
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

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct MigrationReporter<'m> {
    last_event: Option<SpeciationEvent>,

    migrations: &'m mut usize,
}

impl<'m> MigrationReporter<'m> {
    pub fn new(migrations: &'m mut usize) -> Self {
        Self {
            last_event: None,
            migrations,
        }
    }
}

impl<'m> ReporterContext for MigrationReporter<'m> {
    type Reporter = Self;

    fn try_build<KeepSpeciation: Boolean, KeepDispersal: Boolean, KeepProgress: Boolean>(
        self,
    ) -> anyhow::Result<FilteredReporter<Self::Reporter, KeepSpeciation, KeepDispersal, KeepProgress>>
    {
        Ok(FilteredReporter::from(self))
    }
}

impl<'m> Reporter for MigrationReporter<'m> {
    impl_report!(speciation(&mut self, event: Unused) -> Used {
        event.use_in(|event| {
            if Some(event) == self.last_event.as_ref() {
                return;
            }

            self.last_event = Some(event.clone());

            *self.migrations += 1;
        })
    });

    impl_report!(dispersal(&mut self, event: Unused) -> Unused {
        event.ignore()
    });

    impl_report!(progress(&mut self, remaining: Unused) -> Unused {
        remaining.ignore()
    });
}

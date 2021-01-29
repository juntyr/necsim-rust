use crate::reporter::ReporterContext;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
pub trait NonSpatialMigrationSimulation {
    type Error;

    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&local_migration_probability_per_generation),
        "0.0 <= local_migration_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&meta_speciation_probability_per_generation),
        "0.0 <= meta_speciation_probability_per_generation <= 1.0"
    )]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&sample_percentage),
        "0.0 <= sample_percentage <= 1.0"
    )]
    fn simulate<P: ReporterContext>(
        local_area_deme: ((u32, u32), u32),
        meta_area_deme: ((u32, u32), u32),
        local_migration_probability_per_generation: f64,
        meta_speciation_probability_per_generation: f64,
        sample_percentage: f64,
        seed: u64,
        reporter_context: P,
    ) -> Result<(f64, u64), Self::Error>;
}
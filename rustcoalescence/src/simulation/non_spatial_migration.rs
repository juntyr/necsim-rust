use anyhow::{Context, Result};

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

// #[cfg(feature = "necsim-cuda")]
// use necsim_cuda::CudaSimulation;
//
// #[cfg(feature = "necsim-gillespie")]
// use necsim_gillespie::GillespieSimulation;
//
// #[cfg(feature = "necsim-skipping-gillespie")]
// use necsim_skipping_gillespie::SkippingGillespieSimulation;

// #[cfg(feature = "necsim-independent")]
// use necsim_independent::IndependentSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_no_std::simulation::non_spatial_migration::NonSpatialMigrationSimulation;

#[allow(unused_imports)]
use crate::args::{Algorithm, CommonArgs, NonSpatialMigrationArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<P: ReporterContext>(
    common_args: &CommonArgs,
    non_spatial_migration_args: &NonSpatialMigrationArgs,
    reporter_context: P,
) -> Result<(f64, u64)> {
    println!(
        "Setting up the non-spatial-migration {:?} coalescence algorithm ...",
        common_args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    #[allow(clippy::map_err_ignore)]
    let result: Result<(f64, u64)> = match common_args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            (
                *non_spatial_migration_args.local_area(),
                *non_spatial_migration_args.local_deme(),
            ),
            (
                *non_spatial_migration_args.meta_area(),
                *non_spatial_migration_args.meta_deme(),
            ),
            *non_spatial_migration_args.migration_probability_per_generation(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
            (),
        )
        .map_err(|_| unreachable!("Non-Spatial ClassicalSimulation can never fail.")),
        // #[cfg(feature = "necsim-gillespie")]
        // Algorithm::Gillespie => GillespieSimulation::simulate(
        // non_spatial_migration_args.local_area(),
        // non_spatial_migration_args.local_deme(),
        // non_spatial_migration_args.meta_area(),
        // non_spatial_migration_args.meta_deme(),
        // non_spatial_migration_args.migration_probability_per_generation(),
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // (),
        // )
        // .map_err(|_| unreachable!("Non-Spatial GillespieSimulation can never fail.")),
        // #[cfg(feature = "necsim-skipping-gillespie")]
        // Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
        // non_spatial_migration_args.local_area(),
        // non_spatial_migration_args.local_deme(),
        // non_spatial_migration_args.meta_area(),
        // non_spatial_migration_args.meta_deme(),
        // non_spatial_migration_args.migration_probability_per_generation(),
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // (),
        // )
        // .map_err(|_| unreachable!("Non-Spatial SkippingGillespieSimulation can never fail.")),
        // #[cfg(feature = "necsim-cuda")]
        // Algorithm::CUDA => CudaSimulation::simulate(
        // non_spatial_migration_args.local_area(),
        // non_spatial_migration_args.local_deme(),
        // non_spatial_migration_args.meta_area(),
        // non_spatial_migration_args.meta_deme(),
        // non_spatial_migration_args.migration_probability_per_generation(),
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // (),
        // ),
        // #[cfg(feature = "necsim-independent")]
        // Algorithm::Independent => IndependentSimulation::simulate(
        // non_spatial_migration_args.local_area(),
        // non_spatial_migration_args.local_deme(),
        // non_spatial_migration_args.meta_area(),
        // non_spatial_migration_args.meta_deme(),
        // non_spatial_migration_args.migration_probability_per_generation(),
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // (),
        // )
        // .map_err(|_| unreachable!("Non-Spatial IndependentSimulation can never fail.")),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            "Failed to run the non-spatial-migration simulation with local area {:?} and deme \
             {:?}, meta area {:?} and deme {:?}, local migration probability {:?} and meta \
             speciation probability {:?}.",
            non_spatial_migration_args.local_area(),
            non_spatial_migration_args.local_deme(),
            non_spatial_migration_args.meta_area(),
            non_spatial_migration_args.meta_deme(),
            non_spatial_migration_args.migration_probability_per_generation(),
            common_args.speciation_probability_per_generation()
        )
    })
}

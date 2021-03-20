use anyhow::{Context, Result};

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_no_std::simulation::spatially_implicit::SpatiallyImplicitSimulation;

use necsim_impls_no_std::partitioning::LocalPartition;

#[allow(unused_imports)]
use crate::args::{Algorithm, CommonArgs, SpatiallyImplicitArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<R: ReporterContext, P: LocalPartition<R>>(
    common_args: CommonArgs,
    spatially_implicit_args: SpatiallyImplicitArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    info!(
        "Setting up the spatially-implicit {} coalescence algorithm ...",
        common_args.algorithm
    );

    #[allow(clippy::match_single_binding)]
    #[allow(clippy::map_err_ignore)]
    let result: Result<(f64, u64)> = match common_args.algorithm {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            spatially_implicit_args.dynamic_meta,
            (
                spatially_implicit_args.local_area,
                spatially_implicit_args.local_deme,
            ),
            (
                spatially_implicit_args.meta_area,
                spatially_implicit_args.meta_deme,
            ),
            spatially_implicit_args
                .migration_probability_per_generation
                .get(),
            common_args.speciation_probability_per_generation.get(),
            common_args.sample_percentage.get(),
            common_args.seed,
            local_partition,
            (),
        )
        .map_err(|_| unreachable!("Spatially-Implicit ClassicalSimulation can never fail.")),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| "Failed to run the spatially-implicit simulation.")
}

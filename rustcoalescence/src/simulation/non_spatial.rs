use anyhow::{Context, Result};

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

#[cfg(feature = "necsim-cuda")]
use necsim_cuda::CudaSimulation;

#[cfg(feature = "necsim-gillespie")]
use necsim_gillespie::GillespieSimulation;

#[cfg(feature = "necsim-skipping-gillespie")]
use necsim_skipping_gillespie::SkippingGillespieSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_std::simulation::non_spatial::NonSpatialSimulation;

#[allow(unused_imports)]
use crate::args::{Algorithm, CommonArgs, NonSpatialArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<P: ReporterContext>(
    common_args: &CommonArgs,
    non_spatial_args: &NonSpatialArgs,
    reporter_context: P,
) -> Result<(f64, u64)> {
    println!(
        "Setting up the non-spatial {:?} coalescence algorithm ...",
        common_args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    let result: Result<(f64, u64)> = match common_args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            *non_spatial_args.area(),
            *non_spatial_args.deme(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        // #[cfg(feature = "necsim-gillespie")]
        // Algorithm::Gillespie => GillespieSimulation::simulate(
        // habitat,
        // &dispersal,
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // ),
        // #[cfg(feature = "necsim-skipping-gillespie")]
        // Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
        // habitat,
        // &dispersal,
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // ),
        // #[cfg(feature = "necsim-cuda")]
        // Algorithm::CUDA => CudaSimulation::simulate(
        // habitat,
        // &dispersal,
        // common_args.speciation_probability_per_generation(),
        // common_args.sample_percentage(),
        // common_args.seed(),
        // reporter_context,
        // ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            "Failed to run the non-spatial simulation with area {:?} and deme {:?}.",
            non_spatial_args.area(),
            non_spatial_args.deme()
        )
    })
}

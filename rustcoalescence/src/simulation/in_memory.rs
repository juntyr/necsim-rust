use anyhow::{Context, Result};
use array2d::Array2D;

#[cfg(feature = "necsim-classical")]
use necsim_classical::ClassicalSimulation;

#[cfg(feature = "necsim-cuda")]
use necsim_cuda::CudaSimulation;

#[cfg(feature = "necsim-gillespie")]
use necsim_gillespie::GillespieSimulation;

#[cfg(feature = "necsim-skipping-gillespie")]
use necsim_skipping_gillespie::SkippingGillespieSimulation;

#[cfg(feature = "necsim-independent")]
use necsim_independent::IndependentSimulation;

use necsim_impls_no_std::reporter::ReporterContext;
#[allow(unused_imports)]
use necsim_impls_no_std::simulation::in_memory::InMemorySimulation;

#[allow(unused_imports)]
use crate::args::{Algorithm, CommonArgs, InMemoryArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<P: ReporterContext>(
    common_args: &CommonArgs,
    in_memory_args: &InMemoryArgs,
    habitat: &Array2D<u32>,
    dispersal: &Array2D<f64>,
    reporter_context: P,
) -> Result<(f64, u64)> {
    println!(
        "Setting up the in-memory {:?} coalescence algorithm ...",
        common_args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    let result: Result<(f64, u64)> = match common_args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            habitat,
            &dispersal,
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[cfg(feature = "necsim-gillespie")]
        Algorithm::Gillespie => GillespieSimulation::simulate(
            habitat,
            &dispersal,
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[cfg(feature = "necsim-skipping-gillespie")]
        Algorithm::SkippingGillespie => SkippingGillespieSimulation::simulate(
            habitat,
            &dispersal,
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[cfg(feature = "necsim-cuda")]
        Algorithm::CUDA => CudaSimulation::simulate(
            habitat,
            &dispersal,
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[cfg(feature = "necsim-independent")]
        Algorithm::Independent => IndependentSimulation::simulate(
            habitat,
            &dispersal,
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            reporter_context,
        ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            "Failed to run the in-memory simulation with the habitat map {:?} and the dispersal \
             map {:?}.",
            in_memory_args.dispersal_map(),
            in_memory_args.habitat_map()
        )
    })
}

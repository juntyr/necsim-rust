use anyhow::{Context, Result};

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
use necsim_impls_no_std::simulation::almost_infinite::AlmostInfiniteSimulation;

use necsim_impls_no_std::partitioning::LocalPartition;

#[allow(unused_imports)]
use crate::args::{Algorithm, AlmostInfiniteArgs, CommonArgs};

#[allow(unreachable_code)]
#[allow(unused_variables)]
#[allow(clippy::needless_pass_by_value)]
pub fn simulate<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    almost_infinite_args: &AlmostInfiniteArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    info!(
        "Setting up the almost-infinite {:?} coalescence algorithm ...",
        common_args.algorithm()
    );

    #[allow(clippy::match_single_binding)]
    #[allow(clippy::map_err_ignore)]
    let result: Result<(f64, u64)> = match common_args.algorithm() {
        #[cfg(feature = "necsim-classical")]
        Algorithm::Classical => ClassicalSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            local_partition,
            (),
        )
        .map_err(|_| unreachable!("Almost-Infinite ClassicalSimulation can never fail.")),
        #[cfg(feature = "necsim-gillespie")]
        Algorithm::Gillespie => GillespieSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            local_partition,
            (),
        )
        .map_err(|_| unreachable!("Almost-Infinite GillespieSimulation can never fail.")),
        #[cfg(feature = "necsim-skipping-gillespie")]
        Algorithm::SkippingGillespie(auxiliary) => SkippingGillespieSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            local_partition,
            *auxiliary,
        )
        .map_err(|_| unreachable!("Almost-Infinite SkippingGillespieSimulation can never fail.")),
        #[cfg(feature = "necsim-cuda")]
        Algorithm::Cuda(auxiliary) => CudaSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            local_partition,
            *auxiliary,
        ),
        #[cfg(feature = "necsim-independent")]
        Algorithm::Independent(auxiliary) => IndependentSimulation::simulate(
            *almost_infinite_args.radius(),
            *almost_infinite_args.sigma(),
            *common_args.speciation_probability_per_generation(),
            *common_args.sample_percentage(),
            *common_args.seed(),
            local_partition,
            *auxiliary,
        ),
        #[allow(unreachable_patterns)]
        _ => anyhow::bail!("rustcoalescence does not support the selected algorithm"),
    };

    result.with_context(|| {
        format!(
            "Failed to run the almost-infinite simulation with radius {:?} and sigma {:?}.",
            almost_infinite_args.radius(),
            almost_infinite_args.sigma()
        )
    })
}

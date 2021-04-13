use anyhow::Result;

use rustcoalescence_algorithms::Algorithm;

#[cfg(feature = "rustcoalescence-algorithms-cuda")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-independent")]
use rustcoalescence_algorithms_independent::IndependentAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-monolithic")]
use rustcoalescence_algorithms_monolithic::{
    classical::ClassicalAlgorithm, gillespie::GillespieAlgorithm,
    skipping_gillespie::SkippingGillespieAlgorithm,
};

use necsim_impls_no_std::{
    cogs::origin_sampler::pre_sampler::OriginPreSampler, partitioning::LocalPartition,
    reporter::ReporterContext,
};
use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteScenario, non_spatial::NonSpatialScenario,
    spatially_explicit::SpatiallyExplicitScenario, spatially_implicit::SpatiallyImplicitScenario,
    Scenario,
};

use crate::args::{Algorithm as AlgorithmArgs, CommonArgs, Scenario as ScenarioArgs};

#[cfg(not(feature = "necsim-partitioning-mpi"))]
pub mod monolithic;
#[cfg(feature = "necsim-partitioning-mpi")]
pub mod mpi;

#[macro_use]
mod r#impl;

#[allow(
    clippy::module_name_repetitions,
    clippy::too_many_lines,
    clippy::boxed_local
)]
pub fn simulate_with_logger<R: ReporterContext, P: LocalPartition<R>>(
    mut local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
) -> Result<()> {
    if local_partition.get_number_of_partitions().get() <= 1 {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            local_partition.get_number_of_partitions().get()
        );
    }

    let pre_sampler = OriginPreSampler::all().percentage(common_args.sample_percentage.get());

    let (time, steps): (f64, u64) = crate::match_scenario_algorithm!(
        (common_args.algorithm, scenario => scenario)
    {
        #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
        AlgorithmArgs::Classical(algorithm_args) => {
            ClassicalAlgorithm::initialise_and_simulate(
                algorithm_args,
                common_args.seed,
                scenario,
                pre_sampler,
                &mut *local_partition,
            )
            .into_ok()
        },
        #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
        AlgorithmArgs::Gillespie(algorithm_args) => {
            GillespieAlgorithm::initialise_and_simulate(
                algorithm_args,
                common_args.seed,
                scenario,
                pre_sampler,
                &mut *local_partition,
            )
            .into_ok()
        },
        #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
        AlgorithmArgs::SkippingGillespie(algorithm_args) => {
            SkippingGillespieAlgorithm::initialise_and_simulate(
                algorithm_args,
                common_args.seed,
                scenario,
                pre_sampler,
                &mut *local_partition,
            )
            .into_ok()
        },
        #[cfg(feature = "rustcoalescence-algorithms-independent")]
        AlgorithmArgs::Independent(algorithm_args) => {
            IndependentAlgorithm::initialise_and_simulate(
                algorithm_args,
                common_args.seed,
                scenario,
                pre_sampler,
                &mut *local_partition,
            )
            .into_ok()
        },
        #[cfg(feature = "rustcoalescence-algorithms-cuda")]
        AlgorithmArgs::Cuda(algorithm_args) => {
            CudaAlgorithm::initialise_and_simulate(
                algorithm_args,
                common_args.seed,
                scenario,
                pre_sampler,
                &mut *local_partition,
            )?
        }
        <=>
        ScenarioArgs::SpatiallyExplicit(scenario_args) => {
            SpatiallyExplicitScenario::initialise(
                scenario_args,
                common_args.speciation_probability_per_generation,
            )?
        },
        ScenarioArgs::NonSpatial(scenario_args) => {
            NonSpatialScenario::initialise(
                scenario_args,
                common_args.speciation_probability_per_generation,
            )
            .into_ok()
        },
        ScenarioArgs::AlmostInfinite(scenario_args) => {
            AlmostInfiniteScenario::initialise(
                scenario_args,
                common_args.speciation_probability_per_generation,
            )
            .into_ok()
        },
        ScenarioArgs::SpatiallyImplicit(scenario_args) => {
            SpatiallyImplicitScenario::initialise(
                scenario_args,
                common_args.speciation_probability_per_generation,
            )
            .into_ok()
        }
    });

    if log::log_enabled!(log::Level::Info) {
        eprintln!("\n");
        eprintln!("{:=^80}", " Reporter Summary ");
        eprintln!();
    }
    local_partition.finalise_reporting();
    if log::log_enabled!(log::Level::Info) {
        eprintln!();
        eprintln!("{:=^80}", " Reporter Summary ");
        eprintln!();
    }

    info!(
        "The simulation finished at time {} after {} steps.\n",
        time, steps
    );

    Ok(())
}

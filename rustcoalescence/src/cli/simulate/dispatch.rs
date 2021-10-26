use anyhow::{Context, Result};

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

use necsim_core::{
    cogs::SeedableRng,
    reporter::{
        boolean::{Boolean, False, True},
        Reporter,
    },
};
use necsim_core_bond::NonNegativeF64;
use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteScenario, non_spatial::NonSpatialScenario,
    spatially_explicit::SpatiallyExplicitScenario, spatially_implicit::SpatiallyImplicitScenario,
    Scenario,
};

use crate::args::{
    Algorithm as AlgorithmArgs, CommonArgs, Rng as RngArgs, Scenario as ScenarioArgs,
};

pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>>(
    local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
) -> Result<()> {
    Dispatcher::simulate_with_logger(local_partition, common_args, scenario)
}

trait SimulateSealedBooleanDispatch<
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
    ReportProgress: Boolean,
>
{
    fn simulate_with_logger<
        R: Reporter<
            ReportSpeciation = ReportSpeciation,
            ReportDispersal = ReportDispersal,
            ReportProgress = ReportProgress,
        >,
        P: LocalPartition<R>,
    >(
        local_partition: Box<P>,
        common_args: CommonArgs,
        scenario: ScenarioArgs,
    ) -> Result<()>;
}

struct Dispatcher;

macro_rules! initialise_and_simulate {
    (
        $algorithm:ident($common_args:ident, $algorithm_args:ident, $scenario:ident, $local_partition:ident)
    ) => {
        $algorithm::initialise_and_simulate(
            $algorithm_args,
            match $common_args.rng {
                RngArgs::Seed(seed) => SeedableRng::seed_from_u64(seed),
                RngArgs::State(state) => bincode::deserialize(&state)
                    .context("Failed to initialise the RNG from the given state")?,
            },
            $scenario,
            OriginPreSampler::all().percentage($common_args.sample_percentage.get()),
            &mut *$local_partition,
        )
    };
}

macro_rules! impl_sealed_dispatch {
    ($report_speciation:ty, $report_dispersal:ty, $report_progress:ty) => {
        impl SimulateSealedBooleanDispatch<
            $report_speciation, $report_dispersal, $report_progress,
        > for Dispatcher {
            #[allow(clippy::too_many_lines, clippy::boxed_local)]
            fn simulate_with_logger<R: Reporter<
                ReportSpeciation = $report_speciation,
                ReportDispersal = $report_dispersal,
                ReportProgress = $report_progress
            >, P: LocalPartition<R>>(
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

                let (time, steps): (NonNegativeF64, u64) = crate::match_scenario_algorithm!(
                    (common_args.algorithm, scenario => scenario)
                {
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::Classical(algorithm_args) => { initialise_and_simulate!(
                        ClassicalAlgorithm(common_args, algorithm_args, scenario, local_partition)
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::Gillespie(algorithm_args) => { initialise_and_simulate!(
                        GillespieAlgorithm(common_args, algorithm_args, scenario, local_partition)
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::SkippingGillespie(algorithm_args) => { initialise_and_simulate!(
                        SkippingGillespieAlgorithm(common_args, algorithm_args, scenario, local_partition)
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-independent")]
                    AlgorithmArgs::Independent(algorithm_args) => { initialise_and_simulate!(
                        IndependentAlgorithm(common_args, algorithm_args, scenario, local_partition)
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
                    AlgorithmArgs::Cuda(algorithm_args) => { initialise_and_simulate!(
                        CudaAlgorithm(common_args, algorithm_args, scenario, local_partition)
                    )? }
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
                    println!("\n");
                    println!("{:=^80}", " Reporter Summary ");
                    println!();
                }
                local_partition.finalise_reporting();
                if log::log_enabled!(log::Level::Info) {
                    println!();
                    println!("{:=^80}", " Reporter Summary ");
                    println!();
                }

                info!(
                    "The simulation finished at time {} after {} steps.\n",
                    time.get(),
                    steps
                );

                Ok(())
            }
        }
    };
}

impl_sealed_dispatch!(False, False, False);
impl_sealed_dispatch!(False, True, False);
impl_sealed_dispatch!(True, False, False);
impl_sealed_dispatch!(True, True, False);
impl_sealed_dispatch!(False, False, True);
impl_sealed_dispatch!(False, True, True);
impl_sealed_dispatch!(True, False, True);
impl_sealed_dispatch!(True, True, True);

impl<ReportSpeciation: Boolean, ReportDispersal: Boolean, ReportProgress: Boolean>
    SimulateSealedBooleanDispatch<ReportSpeciation, ReportDispersal, ReportProgress>
    for Dispatcher
{
    default fn simulate_with_logger<
        R: Reporter<
            ReportSpeciation = ReportSpeciation,
            ReportDispersal = ReportDispersal,
            ReportProgress = ReportProgress,
        >,
        P: LocalPartition<R>,
    >(
        _local_partition: Box<P>,
        _common_args: CommonArgs,
        _scenario: ScenarioArgs,
    ) -> Result<()> {
        // Safety:
        // - `Boolean` is sealed and must be either `False` or `True`
        // - `SimulateSealedBooleanDispatch` is specialised for all combinations
        unsafe { std::hint::unreachable_unchecked() }
    }
}

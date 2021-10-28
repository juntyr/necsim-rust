use anyhow::{Context, Result};
use bincode::Options;
use tiny_keccak::{Hasher, Keccak};

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
    cogs::{MathsCore, SeedableRng},
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
    Algorithm as AlgorithmArgs, Base32String, CommonArgs, Rng as RngArgs, Scenario as ScenarioArgs,
};

pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
    local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    post_validation: V,
    pre_launch: L,
) -> Result<()> {
    Dispatcher::simulate_with_logger(
        local_partition,
        common_args,
        scenario,
        algorithm,
        post_validation,
        pre_launch,
    )
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
        V: FnOnce(),
        L: FnOnce(),
    >(
        local_partition: Box<P>,
        common_args: CommonArgs,
        scenario: ScenarioArgs,
        algorithm: AlgorithmArgs,
        post_validation: V,
        pre_launch: L,
    ) -> Result<()>;
}

fn seed_with_sponge<M: MathsCore, G: SeedableRng<M>>(bytes: &[u8]) -> G {
    let mut seed = G::Seed::default();

    let mut sponge = Keccak::v256();
    sponge.update(bytes);
    sponge.finalize(seed.as_mut());

    G::from_seed(seed)
}

fn initialise_rng<M: MathsCore, G: SeedableRng<M>>(args: RngArgs) -> Result<(G, impl FnOnce())> {
    #[derive(Debug)]
    enum Rng {
        Seed(u64),
        Sponge(Base32String),
        State(Base32String),
    }

    let (rng, args) = match args {
        RngArgs::Entropy => {
            let mut entropy = G::Seed::default();
            getrandom::getrandom(entropy.as_mut())
                .map_err(|err| anyhow::anyhow!("simulate.rng.Entropy: {}", err))
                .context("Failed to validate the simulate subcommand arguments.")?;

            (
                seed_with_sponge(entropy.as_mut()),
                Rng::Sponge(Base32String::new(entropy.as_mut())),
            )
        },
        RngArgs::Seed(seed) => (G::seed_from_u64(seed), Rng::Seed(seed)),
        RngArgs::Sponge(state) => (seed_with_sponge(&state), Rng::Sponge(state)),
        RngArgs::State(state) => {
            let rng = bincode::options()
                .deserialize(&state)
                .map_err(|_| anyhow::anyhow!("simulate.rng.State: invalid RNG state {}", state))
                .context("Failed to validate the simulate subcommand arguments.")?;

            (rng, Rng::State(state))
        },
        RngArgs::StateElseSponge(state) => match bincode::options().deserialize(&state) {
            Ok(rng) => (rng, Rng::State(state)),
            Err(_) => (seed_with_sponge(&state), Rng::Sponge(state)),
        },
    };

    Ok((rng, move || {
        info!("Initialised the RNG using the {:?} method.", args);
    }))
}

macro_rules! initialise_and_simulate {
    (
        $algorithm:ident(
            $common_args:ident,
            $algorithm_args:ident,
            $scenario:ident,
            $local_partition:ident,
            $post_validation:ident,
            $pre_launch:ident
        )
    ) => {{
        let (rng, rng_info) = initialise_rng($common_args.rng)?;

        ($post_validation)();
        (rng_info)();
        ($pre_launch)();

        $algorithm::initialise_and_simulate(
            $algorithm_args,
            rng,
            $scenario,
            OriginPreSampler::all().percentage($common_args.sample_percentage.get()),
            &mut *$local_partition,
        )
    }};
}

struct Dispatcher;

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
            >, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
                mut local_partition: Box<P>,
                common_args: CommonArgs,
                scenario: ScenarioArgs,
                algorithm: AlgorithmArgs,
                post_validation: V,
                pre_launch_orig: L,
            ) -> Result<()> {
                let number_of_partitions = local_partition.get_number_of_partitions().get();

                let pre_launch = || {
                    (pre_launch_orig)();

                    if number_of_partitions <= 1 {
                        info!("The simulation will be run in monolithic mode.");
                    } else {
                        info!(
                            "The simulation will be distributed across {} partitions.",
                            number_of_partitions
                        );
                    }
                };

                let (time, steps): (NonNegativeF64, u64) = crate::match_scenario_algorithm!(
                    (algorithm, scenario => scenario)
                {
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::Classical(algorithm_args) => { initialise_and_simulate!(
                        ClassicalAlgorithm(
                            common_args, algorithm_args, scenario, local_partition,
                            post_validation, pre_launch
                        )
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::Gillespie(algorithm_args) => { initialise_and_simulate!(
                        GillespieAlgorithm(
                            common_args, algorithm_args, scenario, local_partition,
                            post_validation, pre_launch
                        )
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
                    AlgorithmArgs::SkippingGillespie(algorithm_args) => { initialise_and_simulate!(
                        SkippingGillespieAlgorithm(
                            common_args, algorithm_args, scenario, local_partition,
                            post_validation, pre_launch
                        )
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-independent")]
                    AlgorithmArgs::Independent(algorithm_args) => { initialise_and_simulate!(
                        IndependentAlgorithm(
                            common_args, algorithm_args, scenario, local_partition,
                            post_validation, pre_launch
                        )
                    ).into_ok() },
                    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
                    AlgorithmArgs::Cuda(algorithm_args) => { initialise_and_simulate!(
                        CudaAlgorithm(
                            common_args, algorithm_args, scenario, local_partition,
                            post_validation, pre_launch
                        )
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
        V: FnOnce(),
        L: FnOnce(),
    >(
        _local_partition: Box<P>,
        _common_args: CommonArgs,
        _scenario: ScenarioArgs,
        _algorithm: AlgorithmArgs,
        _post_validation: V,
        _pre_launch: L,
    ) -> Result<()> {
        // Safety:
        // - `Boolean` is sealed and must be either `False` or `True`
        // - `SimulateSealedBooleanDispatch` is specialised for all combinations
        unsafe { std::hint::unreachable_unchecked() }
    }
}

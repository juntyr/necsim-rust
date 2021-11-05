use anyhow::{Context, Result};
use bincode::Options;
use tiny_keccak::{Hasher, Keccak};

use rustcoalescence_algorithms::Algorithm;

#[cfg(feature = "rustcoalescence-algorithms-cuda")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-gillespie")]
use rustcoalescence_algorithms_gillespie::{
    classical::ClassicalAlgorithm, event_skipping::EventSkippingAlgorithm,
};
#[cfg(feature = "rustcoalescence-algorithms-independent")]
use rustcoalescence_algorithms_independent::IndependentAlgorithm;

use necsim_core::{
    cogs::{MathsCore, SeedableRng},
    reporter::Reporter,
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
    Algorithm as AlgorithmArgs, Base32String, CommonArgs, Pause as PauseArgs, Rng as RngArgs,
    Scenario as ScenarioArgs,
};

pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
    mut local_partition: P,
    common_args: CommonArgs,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    pause: Option<PauseArgs>,
    post_validation: V,
    pre_launch: L,
) -> Result<(NonNegativeF64, u64)> {
    let pause_before = pause.map(|pause| pause.before);

    let (time, steps) = crate::match_scenario_algorithm!(
        (algorithm, scenario => scenario)
    {
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::Classical(algorithm_args) => { initialise_and_simulate::<ClassicalAlgorithm, _, R, P, V, L>(
            common_args, algorithm_args, scenario, pause_before, &mut local_partition,
            post_validation, pre_launch
        ) },
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::EventSkipping(algorithm_args) => { initialise_and_simulate::<EventSkippingAlgorithm, _, R, P, V, L>(
            common_args, algorithm_args, scenario, pause_before, &mut local_partition,
            post_validation, pre_launch
        ) },
        #[cfg(feature = "rustcoalescence-algorithms-independent")]
        AlgorithmArgs::Independent(algorithm_args) => { initialise_and_simulate::<IndependentAlgorithm, _, R, P, V, L>(
            common_args, algorithm_args, scenario, pause_before, &mut local_partition,
            post_validation, pre_launch
        ) },
        #[cfg(feature = "rustcoalescence-algorithms-cuda")]
        AlgorithmArgs::Cuda(algorithm_args) => { initialise_and_simulate::<CudaAlgorithm, _, R, P, V, L>(
            common_args, algorithm_args, scenario, pause_before, &mut local_partition,
            post_validation, pre_launch
        ) }
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
    })?;

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

    Ok((time, steps))
}

fn initialise_and_simulate<
    A: Algorithm<O, R, P>,
    O: Scenario<A::MathsCore, A::Rng>,
    R: Reporter,
    P: LocalPartition<R>,
    V: FnOnce(),
    L: FnOnce(),
>(
    common_args: CommonArgs,
    algorithm_args: A::Arguments,
    scenario: O,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
    post_validation: V,
    pre_launch: L,
) -> anyhow::Result<(NonNegativeF64, u64)>
where
    Result<(NonNegativeF64, u64), A::Error>: anyhow::Context<(NonNegativeF64, u64), A::Error>,
{
    let (rng, rng_info) = initialise_rng(common_args.rng)?;

    (post_validation)();
    (rng_info)();
    (pre_launch)();

    A::initialise_and_simulate(
        algorithm_args,
        rng,
        scenario,
        OriginPreSampler::all().percentage(common_args.sample_percentage.get()),
        pause_before,
        local_partition,
    )
    .context("Failed to perform the simulation.")
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

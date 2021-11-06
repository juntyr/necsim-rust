use anyhow::{Context, Result};
use serde::Deserialize;
use tiny_keccak::{Hasher, Keccak};

use rustcoalescence_algorithms::{Algorithm, AlgorithmResult};

#[cfg(feature = "rustcoalescence-algorithms-cuda")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-gillespie")]
use rustcoalescence_algorithms_gillespie::{
    classical::ClassicalAlgorithm, event_skipping::EventSkippingAlgorithm,
};
#[cfg(feature = "rustcoalescence-algorithms-independent")]
use rustcoalescence_algorithms_independent::IndependentAlgorithm;

use necsim_core::{
    cogs::{MathsCore, RngCore, SeedableRng},
    reporter::{boolean::Boolean, Reporter},
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveUnitF64};
use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteScenario, non_spatial::NonSpatialScenario,
    spatially_explicit::SpatiallyExplicitScenario, spatially_implicit::SpatiallyImplicitScenario,
    Scenario,
};

use crate::args::{
    parse::{ron_config, try_parse},
    Algorithm as AlgorithmArgs, Base32String, Rng as RngArgs, Scenario as ScenarioArgs,
};

use super::{BufferingPartialSimulateArgs, BufferingSimulateArgs, ResumingRng, SimulationResult};

#[allow(clippy::too_many_arguments)]
pub(super) fn simulate_with_logger<R: Reporter, P: LocalPartition<R>>(
    mut local_partition: P,
    speciation_probability_per_generation: PositiveUnitF64,
    sample_percentage: ClosedUnitF64,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    pause_before: Option<NonNegativeF64>,
    ron_args: &str,
    partial_simulate_args: BufferingPartialSimulateArgs,
) -> Result<SimulationResult> {
    let result = crate::match_scenario_algorithm!(
        (algorithm, scenario => scenario)
    {
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::Classical(algorithm_args) => {
            initialise_and_simulate::<ClassicalAlgorithm, _, R, P>(
                &mut local_partition, sample_percentage, algorithm_args,
                scenario, pause_before, ron_args, partial_simulate_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::EventSkipping(algorithm_args) => {
            initialise_and_simulate::<EventSkippingAlgorithm, _, R, P>(
                &mut local_partition, sample_percentage, algorithm_args,
                scenario, pause_before, ron_args, partial_simulate_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-independent")]
        AlgorithmArgs::Independent(algorithm_args) => {
            initialise_and_simulate::<IndependentAlgorithm, _, R, P>(
                &mut local_partition, sample_percentage, algorithm_args,
                scenario, pause_before, ron_args, partial_simulate_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-cuda")]
        AlgorithmArgs::Cuda(algorithm_args) => {
            initialise_and_simulate::<CudaAlgorithm, _, R, P>(
                &mut local_partition, sample_percentage, algorithm_args,
                scenario, pause_before, ron_args, partial_simulate_args,
            )
        }
        <=>
        ScenarioArgs::SpatiallyExplicit(scenario_args) => {
            SpatiallyExplicitScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )?
        },
        ScenarioArgs::NonSpatial(scenario_args) => {
            NonSpatialScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        },
        ScenarioArgs::AlmostInfinite(scenario_args) => {
            AlmostInfiniteScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        },
        ScenarioArgs::SpatiallyImplicit(scenario_args) => {
            SpatiallyImplicitScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        }
    })?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n");
        println!(
            "{:=^80}",
            if matches!(result, SimulationResult::Done { .. }) {
                " Reporter Summary "
            } else {
                " Partial Reporter Summary "
            }
        );
        println!();
    }
    local_partition.finalise_reporting();
    if log::log_enabled!(log::Level::Info) {
        println!();
        println!(
            "{:=^80}",
            if matches!(result, SimulationResult::Done { .. }) {
                " Reporter Summary "
            } else {
                " Partial Reporter Summary "
            }
        );
        println!();
    }

    Ok(result)
}

#[derive(Deserialize)]
#[serde(bound = "")]
#[serde(rename = "Simulate")]
struct SimulateArgsRngOnly<M: MathsCore, G: RngCore<M>> {
    #[serde(alias = "randomness")]
    rng: RngArgs<M, G>,
}

fn initialise_and_simulate<
    A: Algorithm<O, R, P>,
    O: Scenario<A::MathsCore, A::Rng>,
    R: Reporter,
    P: LocalPartition<R>,
>(
    local_partition: &mut P,
    sample_percentage: ClosedUnitF64,
    algorithm_args: A::Arguments,
    scenario: O,
    pause_before: Option<NonNegativeF64>,
    ron_args: &str,
    partial_simulate_args: BufferingPartialSimulateArgs,
) -> anyhow::Result<SimulationResult>
where
    Result<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>:
        anyhow::Context<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>,
{
    let SimulateArgsRngOnly { rng } = try_parse("simulate", ron_args)?;

    let simulate_args = BufferingSimulateArgs::new(partial_simulate_args, &rng)?;
    let config_str = ron::ser::to_string_pretty(&simulate_args, ron_config())
        .context("Failed to normalise the simulation config.")?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n{:=^80}\n", " Simulation Configuration ");
        println!("{}", config_str.trim_start_matches("Simulate"));
        println!("\n{:=^80}\n", " Simulation Configuration ");
    }

    if local_partition.get_partition().size().get() <= 1 {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            local_partition.get_partition().size().get()
        );
    }

    if P::IsLive::VALUE {
        info!("The simulation will report all events live.");
    } else {
        warn!("The simulation will only report progress events live.");
    }

    let rng = match rng {
        RngArgs::Seed(seed) => SeedableRng::seed_from_u64(seed),
        RngArgs::Sponge(bytes) => {
            let mut seed = <A::Rng as RngCore<A::MathsCore>>::Seed::default();

            let mut sponge = Keccak::v256();
            sponge.update(&bytes);
            sponge.finalize(seed.as_mut());

            RngCore::from_seed(seed)
        },
        RngArgs::State(state) => state.into(),
    };

    let result = A::initialise_and_simulate(
        algorithm_args,
        rng,
        scenario,
        OriginPreSampler::all().percentage(sample_percentage),
        pause_before,
        local_partition,
    )
    .context("Failed to perform the simulation.")?;

    match result {
        AlgorithmResult::Done { time, steps } => Ok(SimulationResult::Done { time, steps }),
        AlgorithmResult::Paused {
            time,
            steps,
            lineages,
            rng: paused_rng,
            ..
        } => {
            let state = bincode::Options::serialize(bincode::options(), &paused_rng)
                .context("Failed to generate config to resume the simulation.")?;

            Ok(SimulationResult::Paused {
                time,
                steps,
                lineages,
                rng: ResumingRng::State(Base32String::new(&state)),
            })
        },
    }
}

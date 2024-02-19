use necsim_core::reporter::Reporter;
use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::AlgorithmDefaults;

#[cfg(feature = "cuda-algorithm")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "gillespie-algorithms")]
use rustcoalescence_algorithms_gillespie::{
    event_skipping::EventSkippingAlgorithm, gillespie::GillespieAlgorithm,
};
#[cfg(feature = "independent-algorithm")]
use rustcoalescence_algorithms_independent::IndependentAlgorithm;

#[cfg(feature = "almost-infinite-scenario")]
use rustcoalescence_scenarios::almost_infinite::AlmostInfiniteScenario;
#[cfg(feature = "non-spatial-scenario")]
use rustcoalescence_scenarios::non_spatial::NonSpatialScenario;
#[cfg(feature = "spatially-explicit-turnover-map-scenario")]
use rustcoalescence_scenarios::spatially_explicit::SpatiallyExplicitTurnoverMapScenario;
#[cfg(feature = "spatially-explicit-uniform-turnover-scenario")]
use rustcoalescence_scenarios::spatially_explicit::SpatiallyExplicitUniformTurnoverScenario;
#[cfg(feature = "spatially-implicit-scenario")]
use rustcoalescence_scenarios::spatially_implicit::SpatiallyImplicitScenario;
#[cfg(feature = "wrapping-noise-scenario")]
use rustcoalescence_scenarios::wrapping_noise::WrappingNoiseScenario;
use rustcoalescence_scenarios::Scenario;

use crate::{
    args::config::{
        algorithm::Algorithm as AlgorithmArgs, sample::Sample as SampleArgs,
        scenario::Scenario as ScenarioArgs,
    },
    cli::simulate::SimulationOutcome,
};

use super::{super::super::BufferingSimulateArgsBuilder, rng};

macro_rules! match_scenario_algorithm {
    (
        ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $($(#[$algmeta:meta])* $algpat:pat => $algcode:block),*
            <=>
            $($(#[$scenmeta:meta])* $scenpat:pat => $scencode:block),*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$algmeta])* $algpat => $algcode),*
                <=>
                $($(#[$scenmeta])* $scenpat => $scencode),*
                <=>
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $(#[$algmeta:meta])* $algpat:pat => $algcode:block,
            $($(#[$algmetarem:meta])* $algpatrem:pat => $algcoderem:block),+
            <=>
            $($(#[$scenmeta:meta])* $scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$algmetarem])* $algpatrem => $algcoderem),+
                <=>
                $($(#[$scenmeta])* $scenpat => $scencode),*
                <=>
                $($tail)*
                $(#[$algmeta])* $algpat => {
                    match $scenario {
                        $($(#[$scenmeta])* $scenpat => {
                            let $algscen = $scencode;
                            $algcode
                        }),*
                    }
                }
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $(#[$algmeta:meta])* $algpat:pat => $algcode:block
            <=>
            $($(#[$scenmeta:meta])* $scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match $algorithm {
            $($tail)*
            $(#[$algmeta])* $algpat => {
                match $scenario {
                    $($(#[$scenmeta])* $scenpat => {
                        let $algscen = $scencode;
                        $algcode
                    }),*
                }
            }
        }
    };
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch<'p, R: Reporter, P: LocalPartition<'p, R>>(
    local_partition: P,

    speciation_probability_per_generation: PositiveUnitF64,
    sample: SampleArgs,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome> {
    match_scenario_algorithm!(
        (algorithm, scenario => scenario)
    {
        #[cfg(feature = "gillespie-algorithms")]
        AlgorithmArgs::Gillespie(algorithm_args) => {
            rng::dispatch::<
                <GillespieAlgorithm as AlgorithmDefaults>::MathsCore,
                GillespieAlgorithm, _, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "gillespie-algorithms")]
        AlgorithmArgs::EventSkipping(algorithm_args) => {
            rng::dispatch::<
                <EventSkippingAlgorithm as AlgorithmDefaults>::MathsCore,
                EventSkippingAlgorithm, _, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "independent-algorithm")]
        AlgorithmArgs::Independent(algorithm_args) => {
            rng::dispatch::<
                <IndependentAlgorithm as AlgorithmDefaults>::MathsCore,
                IndependentAlgorithm, _, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "cuda-algorithm")]
        AlgorithmArgs::Cuda(algorithm_args) => {
            rng::dispatch::<
                <CudaAlgorithm as AlgorithmDefaults>::MathsCore,
                CudaAlgorithm, _, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        }
        <=>
        #[cfg(feature = "spatially-explicit-uniform-turnover-scenario")]
        ScenarioArgs::SpatiallyExplicitUniformTurnover(scenario_args) => {
            SpatiallyExplicitUniformTurnoverScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )?
        },
        #[cfg(feature = "spatially-explicit-turnover-map-scenario")]
        ScenarioArgs::SpatiallyExplicitTurnoverMap(scenario_args) => {
            SpatiallyExplicitTurnoverMapScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )?
        },
        #[cfg(feature = "non-spatial-scenario")]
        ScenarioArgs::NonSpatial(scenario_args) => {
            NonSpatialScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        },
        #[cfg(feature = "almost-infinite-scenario")]
        ScenarioArgs::AlmostInfinite(scenario_args) => {
            AlmostInfiniteScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        },
        #[cfg(feature = "spatially-implicit-scenario")]
        ScenarioArgs::SpatiallyImplicit(scenario_args) => {
            SpatiallyImplicitScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        },
        #[cfg(feature = "wrapping-noise-scenario")]
        ScenarioArgs::WrappingNoise(scenario_args) => {
            WrappingNoiseScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        }
    })
}

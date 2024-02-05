use necsim_core::reporter::Reporter;
use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::AlgorithmDefaults;

#[cfg(feature = "rustcoalescence-algorithms-cuda")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-gillespie")]
use rustcoalescence_algorithms_gillespie::{
    event_skipping::EventSkippingAlgorithm, gillespie::GillespieAlgorithm,
};
#[cfg(feature = "rustcoalescence-algorithms-independent")]
use rustcoalescence_algorithms_independent::IndependentAlgorithm;

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteScenario,
    non_spatial::NonSpatialScenario,
    spatially_explicit::{
        SpatiallyExplicitTurnoverMapScenario, SpatiallyExplicitUniformTurnoverScenario,
    },
    spatially_implicit::SpatiallyImplicitScenario,
    wrapping_noise::WrappingNoiseScenario,
    Scenario,
};

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
        ($algorithm:expr, $scenario:expr => $algscen:ident : $algscenty:ident) {
            $($(#[$meta:meta])* $algpat:pat => $algcode:block),*
            <=>
            $($scenpat:pat => $scencode:block => $scenty:ident),*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen : $algscenty) {
                $($(#[$meta])* $algpat => $algcode),*
                <=>
                $($scenpat => $scencode => $scenty),*
                <=>
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident : $algscenty:ident) {
            $(#[$meta:meta])* $algpat:pat => $algcode:block,
            $($(#[$metarem:meta])* $algpatrem:pat => $algcoderem:block),+
            <=>
            $($scenpat:pat => $scencode:block => $scenty:ident),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen : $algscenty) {
                $($(#[$metarem])* $algpatrem => $algcoderem),+
                <=>
                $($scenpat => $scencode => $scenty),*
                <=>
                $($tail)*
                $(#[$meta])* $algpat => {
                    match $scenario {
                        $($scenpat => {
                            type $algscenty<M, G> = $scenty<M, G>;
                            let $algscen = $scencode;
                            $algcode
                        }),*
                    }
                }
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident : $algscenty:ident) {
            $(#[$meta:meta])* $algpat:pat => $algcode:block
            <=>
            $($scenpat:pat => $scencode:block => $scenty:ident),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match $algorithm {
            $($tail)*
            $(#[$meta])* $algpat => {
                match $scenario {
                    $($scenpat => {
                        type $algscenty<M, G> = $scenty<M, G>;
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
        (algorithm, scenario => scenario: ScenarioTy)
    {
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::Gillespie(algorithm_args) => {
            rng::dispatch::<
                <GillespieAlgorithm as AlgorithmDefaults>::MathsCore,
                <GillespieAlgorithm as AlgorithmDefaults>::Rng<_>,
                GillespieAlgorithm, ScenarioTy<_, _>, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::EventSkipping(algorithm_args) => {
            rng::dispatch::<
                <EventSkippingAlgorithm as AlgorithmDefaults>::MathsCore,
                <EventSkippingAlgorithm as AlgorithmDefaults>::Rng<_>,
                EventSkippingAlgorithm, ScenarioTy<_, _>, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-independent")]
        AlgorithmArgs::Independent(algorithm_args) => {
            rng::dispatch::<
                <IndependentAlgorithm as AlgorithmDefaults>::MathsCore,
                <IndependentAlgorithm as AlgorithmDefaults>::Rng<_>,
                IndependentAlgorithm, ScenarioTy<_, _>, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-cuda")]
        AlgorithmArgs::Cuda(algorithm_args) => {
            rng::dispatch::<
                <CudaAlgorithm as AlgorithmDefaults>::MathsCore,
                <CudaAlgorithm as AlgorithmDefaults>::Rng<_>,
                CudaAlgorithm, ScenarioTy<_, _>, R, P,
            >(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        }
        <=>
        ScenarioArgs::SpatiallyExplicitUniformTurnover(scenario_args) => {
            SpatiallyExplicitUniformTurnoverScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )?
        } => SpatiallyExplicitUniformTurnoverScenario,
        ScenarioArgs::SpatiallyExplicitTurnoverMap(scenario_args) => {
            SpatiallyExplicitTurnoverMapScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )?
        } => SpatiallyExplicitTurnoverMapScenario,
        ScenarioArgs::NonSpatial(scenario_args) => {
            NonSpatialScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        } => NonSpatialScenario,
        ScenarioArgs::AlmostInfinite(scenario_args) => {
            AlmostInfiniteScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        } => AlmostInfiniteScenario,
        ScenarioArgs::SpatiallyImplicit(scenario_args) => {
            SpatiallyImplicitScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        } => SpatiallyImplicitScenario,
        ScenarioArgs::WrappingNoise(scenario_args) => {
            WrappingNoiseScenario::initialise(
                scenario_args,
                speciation_probability_per_generation,
            )
            .into_ok()
        } => WrappingNoiseScenario
    })
}

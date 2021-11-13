use necsim_core::reporter::Reporter;
use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::LocalPartition;

#[cfg(feature = "rustcoalescence-algorithms-cuda")]
use rustcoalescence_algorithms_cuda::CudaAlgorithm;
#[cfg(feature = "rustcoalescence-algorithms-gillespie")]
use rustcoalescence_algorithms_gillespie::{
    classical::ClassicalAlgorithm, event_skipping::EventSkippingAlgorithm,
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
    Scenario,
};

use crate::{
    args::{Algorithm as AlgorithmArgs, Sample as SampleArgs, Scenario as ScenarioArgs},
    cli::simulate::SimulationResult,
};

use super::{super::BufferingSimulateArgsBuilder, rng};

macro_rules! match_scenario_algorithm {
    (
        ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $($(#[$meta:meta])* $algpat:pat => $algcode:block),*
            <=>
            $($scenpat:pat => $scencode:block),*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$meta])* $algpat => $algcode),*
                <=>
                $($scenpat => $scencode),*
                <=>
            }
        }
    };
    (
        impl ($algorithm:expr, $scenario:expr => $algscen:ident) {
            $(#[$meta:meta])* $algpat:pat => $algcode:block,
            $($(#[$metarem:meta])* $algpatrem:pat => $algcoderem:block),+
            <=>
            $($scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match_scenario_algorithm! {
            impl ($algorithm, $scenario => $algscen) {
                $($(#[$metarem])* $algpatrem => $algcoderem),+
                <=>
                $($scenpat => $scencode),*
                <=>
                $($tail)*
                $(#[$meta])* $algpat => {
                    match $scenario {
                        $($scenpat => {
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
            $(#[$meta:meta])* $algpat:pat => $algcode:block
            <=>
            $($scenpat:pat => $scencode:block),*
            <=>
            $($tail:tt)*
        }
    ) => {
        match $algorithm {
            $($tail)*
            $(#[$meta])* $algpat => {
                match $scenario {
                    $($scenpat => {
                        let $algscen = $scencode;
                        $algcode
                    }),*
                }
            }
        }
    };
}

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch<R: Reporter, P: LocalPartition<R>>(
    local_partition: P,

    speciation_probability_per_generation: PositiveUnitF64,
    sample: SampleArgs,
    scenario: ScenarioArgs,
    algorithm: AlgorithmArgs,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationResult> {
    match_scenario_algorithm!(
        (algorithm, scenario => scenario)
    {
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::Classical(algorithm_args) => {
            rng::dispatch::<ClassicalAlgorithm, _, R, P>(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
        AlgorithmArgs::EventSkipping(algorithm_args) => {
            rng::dispatch::<EventSkippingAlgorithm, _, R, P>(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-independent")]
        AlgorithmArgs::Independent(algorithm_args) => {
            rng::dispatch::<IndependentAlgorithm, _, R, P>(
                local_partition, sample, algorithm_args, scenario,
                pause_before, ron_args, normalised_args,
            )
        },
        #[cfg(feature = "rustcoalescence-algorithms-cuda")]
        AlgorithmArgs::Cuda(algorithm_args) => {
            rng::dispatch::<CudaAlgorithm, _, R, P>(
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
        },
        ScenarioArgs::SpatiallyExplicitTurnoverMap(scenario_args) => {
            SpatiallyExplicitTurnoverMapScenario::initialise(
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
    })
}

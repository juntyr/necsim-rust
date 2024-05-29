use necsim_core::{
    cogs::{MathsCore, RngCore},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, LocalPartition, Partitioning as _};

use necsim_partitioning_monolithic::MonolithicLocalPartition;
#[cfg(feature = "mpi-partitioning")]
use necsim_partitioning_mpi::MpiLocalPartition;
use rustcoalescence_algorithms::{result::SimulationOutcome, Algorithm, AlgorithmDispatch};
use rustcoalescence_scenarios::{Scenario, ScenarioCogs};

use crate::args::config::{partitioning::Partitioning, sample::Sample};

use super::{super::super::BufferingSimulateArgsBuilder, info};

#[allow(clippy::too_many_arguments)]
pub(super) fn dispatch<
    M: MathsCore,
    G: RngCore<M>,
    A: AlgorithmDispatch<M, G, O, R>,
    O: Scenario<M, G>,
    R: Reporter,
    P: ReporterContext<Reporter = R>,
>(
    partitioning: Partitioning,
    event_log: Option<EventLogRecorder>,
    reporter_context: P,

    sample: Sample,
    rng: G,
    scenario: ScenarioCogs<M, G, O>,
    algorithm_args: A::Arguments,
    pause_before: Option<NonNegativeF64>,

    normalised_args: &BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome<M, G>>
where
    Result<SimulationOutcome<M, G>, A::Error>: anyhow::Context<SimulationOutcome<M, G>, A::Error>,
{
    let args = (
        sample,
        rng,
        scenario,
        algorithm_args,
        pause_before,
        normalised_args,
    );

    // Initialise the local partition and the simulation
    match partitioning {
        Partitioning::Monolithic(partitioning) => partitioning.with_local_partition(
            reporter_context,
            event_log,
            args,
            |partition, (sample, rng, scenario, algorithm_args, pause_before, normalised_args)| {
                match partition {
                    MonolithicLocalPartition::Live(partition) => {
                        wrap::<M, G, A::Algorithm<'_, _>, O, R, _>(
                            *partition,
                            sample,
                            rng,
                            scenario,
                            algorithm_args,
                            pause_before,
                            normalised_args,
                        )
                    },
                    MonolithicLocalPartition::Recorded(partition) => {
                        wrap::<M, G, A::Algorithm<'_, _>, O, R, _>(
                            *partition,
                            sample,
                            rng,
                            scenario,
                            algorithm_args,
                            pause_before,
                            normalised_args,
                        )
                    },
                }
            },
            fold,
        ),
        #[cfg(feature = "mpi-partitioning")]
        Partitioning::Mpi(partitioning) => partitioning.with_local_partition(
            reporter_context,
            event_log,
            args,
            |partition, (sample, rng, scenario, algorithm_args, pause_before, normalised_args)| {
                match partition {
                    MpiLocalPartition::Root(partition) => {
                        wrap::<M, G, A::Algorithm<'_, _>, O, R, _>(
                            *partition,
                            sample,
                            rng,
                            scenario,
                            algorithm_args,
                            pause_before,
                            normalised_args,
                        )
                    },
                    MpiLocalPartition::Parallel(partition) => {
                        wrap::<M, G, A::Algorithm<'_, _>, O, R, _>(
                            *partition,
                            sample,
                            rng,
                            scenario,
                            algorithm_args,
                            pause_before,
                            normalised_args,
                        )
                    },
                }
            },
            fold,
        ),
        #[cfg(feature = "threads-partitioning")]
        Partitioning::Threads(partitioning) => partitioning.with_local_partition(
            reporter_context,
            event_log,
            args,
            |partition, (sample, rng, scenario, algorithm_args, pause_before, normalised_args)| {
                wrap::<M, G, A::Algorithm<'_, _>, O, R, _>(
                    partition,
                    sample,
                    rng,
                    scenario,
                    algorithm_args,
                    pause_before,
                    normalised_args,
                )
            },
            fold,
        ),
    }
    .and_then(|result| result.map_err(anyhow::Error::msg))
}

fn wrap<
    'p,
    M: MathsCore,
    G: RngCore<M>,
    A: Algorithm<'p, M, G, O, R, P>,
    O: Scenario<M, G>,
    R: Reporter,
    P: LocalPartition<'p, R>,
>(
    local_partition: P,

    sample: Sample,
    rng: G,
    scenario: ScenarioCogs<M, G, O>,
    algorithm_args: A::Arguments,
    pause_before: Option<NonNegativeF64>,

    normalised_args: &BufferingSimulateArgsBuilder,
) -> Result<SimulationOutcome<M, G>, String>
where
    Result<SimulationOutcome<M, G>, A::Error>: anyhow::Context<SimulationOutcome<M, G>, A::Error>,
{
    info::dispatch::<M, G, A::Algorithm<'_, _>, O, R, _>(
        local_partition,
        sample,
        rng,
        scenario,
        algorithm_args,
        pause_before,
        normalised_args,
    )
    .map_err(|err| format!("{err:?}"))
}

fn fold<M: MathsCore, G: RngCore<M>>(
    a: Result<SimulationOutcome<M, G>, String>,
    b: Result<SimulationOutcome<M, G>, String>,
) -> Result<SimulationOutcome<M, G>, String> {
    match (a, b) {
        (
            Ok(SimulationOutcome::Done {
                time: time_a,
                steps: steps_a,
            }),
            Ok(SimulationOutcome::Done {
                time: time_b,
                steps: steps_b,
            }),
        ) => Ok(SimulationOutcome::Done {
            time: time_a.max(time_b),
            steps: steps_a + steps_b,
        }),
        (Ok(SimulationOutcome::Paused { .. }), _) | (_, Ok(SimulationOutcome::Paused { .. })) => {
            Err(String::from(
                "parallel pausing is not yet supported, catching this at simulation completion is \
                 a bug",
            ))
        },
        (Err(err), Ok(_)) | (Ok(_), Err(err)) => Err(err),
        (Err(err_a), Err(err_b)) => Err(format!("{err_a}\n|\n{err_b}")),
    }
}

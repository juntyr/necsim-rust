use necsim_core::{
    cogs::{MathsCore, RngCore},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::{context::ReporterContext, Partitioning as _};

use necsim_partitioning_monolithic::MonolithicLocalPartition;
#[cfg(feature = "necsim-partitioning-mpi")]
use necsim_partitioning_mpi::MpiLocalPartition;
use rustcoalescence_algorithms::{result::SimulationOutcome, AlgorithmDispatch};
use rustcoalescence_scenarios::Scenario;

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
    scenario: O,
    algorithm_args: A::Arguments,
    pause_before: Option<NonNegativeF64>,

    normalised_args: &BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome<M, G>>
where
    Result<SimulationOutcome<M, G>, A::Error>: anyhow::Context<SimulationOutcome<M, G>, A::Error>,
{
    // Initialise the local partition and the simulation
    match partitioning {
        Partitioning::Monolithic(partitioning) => {
            partitioning.with_local_partition(reporter_context, event_log, |partition| {
                match partition {
                    MonolithicLocalPartition::Live(partition) => {
                        info::dispatch::<M, G, A::Algorithm<'_, _>, O, R, _>(
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
                        info::dispatch::<M, G, A::Algorithm<'_, _>, O, R, _>(
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
            })
        },
        #[cfg(feature = "necsim-partitioning-mpi")]
        Partitioning::Mpi(partitioning) => {
            partitioning.with_local_partition(reporter_context, event_log, |partition| {
                match partition {
                    MpiLocalPartition::Root(partition) => {
                        info::dispatch::<M, G, A::Algorithm<'_, _>, O, R, _>(
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
                        info::dispatch::<M, G, A::Algorithm<'_, _>, O, R, _>(
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
            })
        },
    }
    .flatten()
}

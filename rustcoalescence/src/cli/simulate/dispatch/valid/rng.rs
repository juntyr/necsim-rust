use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_partitioning_core::context::ReporterContext;
use tiny_keccak::{Hasher, Keccak};

use rustcoalescence_algorithms::{
    result::SimulationOutcome as AlgorithmOutcome, AlgorithmDispatch,
};

use necsim_core::{
    cogs::{MathsCore, RngCore, SeedableRng},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

use rustcoalescence_scenarios::{Scenario, ScenarioCogs};

use crate::{
    args::config::{
        partitioning::Partitioning,
        rng::{Base32RngState, Rng as RngArgs},
        sample::Sample,
    },
    cli::simulate::parse,
};

use super::{
    super::super::{BufferingSimulateArgsBuilder, SimulationOutcome},
    partitioning,
};

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
    algorithm_args: A::Arguments,
    scenario: ScenarioCogs<M, G, O>,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome>
where
    Result<AlgorithmOutcome<M, G>, A::Error>: anyhow::Context<AlgorithmOutcome<M, G>, A::Error>,
{
    let rng: G = match parse::rng::parse_and_normalise(
        ron_args,
        normalised_args,
        match &partitioning {
            Partitioning::Monolithic(partitioning) => {
                A::get_logical_partition_size(&algorithm_args, partitioning)
            },
            #[cfg(feature = "mpi-partitioning")]
            Partitioning::Mpi(partitioning) => {
                A::get_logical_partition_size(&algorithm_args, partitioning)
            },
            #[cfg(feature = "threads-partitioning")]
            Partitioning::Threads(partitioning) => {
                A::get_logical_partition_size(&algorithm_args, partitioning)
            },
        },
    )? {
        RngArgs::Seed(seed) => SeedableRng::seed_from_u64(seed),
        RngArgs::Sponge(bytes) => {
            let mut seed = G::Seed::default();

            let mut sponge = Keccak::v256();
            sponge.update(&bytes);
            sponge.finalize(seed.as_mut());

            RngCore::from_seed(seed)
        },
        RngArgs::State(state) => state.into(),
    };

    let result = partitioning::dispatch::<M, G, A, O, R, P>(
        partitioning,
        event_log,
        reporter_context,
        sample,
        rng,
        scenario,
        algorithm_args,
        pause_before,
        normalised_args,
    )?;

    match result {
        AlgorithmOutcome::Done { time, steps } => Ok(SimulationOutcome::Done { time, steps }),
        AlgorithmOutcome::Paused {
            time,
            steps,
            lineages,
            rng: paused_rng,
            ..
        } => {
            normalised_args.rng(&RngArgs::State(Base32RngState::from(paused_rng)));

            Ok(SimulationOutcome::Paused {
                time,
                steps,
                lineages,
            })
        },
    }
}

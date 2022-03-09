use tiny_keccak::{Hasher, Keccak};

use rustcoalescence_algorithms::{result::SimulationOutcome as AlgorithmOutcome, Algorithm};

use necsim_core::{
    cogs::{MathsCore, RngCore, SeedableRng},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use crate::{
    args::config::{
        rng::{Base32RngState, Rng as RngArgs},
        sample::Sample,
    },
    cli::simulate::parse,
};

use super::{
    super::super::{BufferingSimulateArgsBuilder, SimulationOutcome},
    info,
};

pub(super) fn dispatch<
    'p,
    M: MathsCore,
    A: Algorithm<'p, M, O, R, P>,
    O: Scenario<M, A::Rng>,
    R: Reporter,
    P: LocalPartition<'p, R>,
>(
    local_partition: P,

    sample: Sample,
    algorithm_args: A::Arguments,
    scenario: O,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome>
where
    Result<AlgorithmOutcome<M, A::Rng>, A::Error>:
        anyhow::Context<AlgorithmOutcome<M, A::Rng>, A::Error>,
{
    let rng: A::Rng = match parse::rng::parse_and_normalise(
        ron_args,
        normalised_args,
        &mut A::get_effective_partition(&algorithm_args, &local_partition),
    )? {
        RngArgs::Seed(seed) => SeedableRng::seed_from_u64(seed),
        RngArgs::Sponge(bytes) => {
            let mut seed = <A::Rng as RngCore<M>>::Seed::default();

            let mut sponge = Keccak::v256();
            sponge.update(&bytes);
            sponge.finalize(seed.as_mut());

            RngCore::from_seed(seed)
        },
        RngArgs::State(state) => state.into(),
    };

    let result = info::dispatch::<M, A, O, R, P>(
        algorithm_args,
        rng,
        scenario,
        sample,
        pause_before,
        local_partition,
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

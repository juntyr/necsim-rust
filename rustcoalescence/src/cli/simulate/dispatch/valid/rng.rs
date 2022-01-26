use tiny_keccak::{Hasher, Keccak};

use rustcoalescence_algorithms::{Algorithm, AlgorithmResult};

use necsim_core::{
    cogs::{RngCore, SeedableRng},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use crate::{
    args::{Base32RngState, Rng as RngArgs, Sample},
    cli::simulate::parse,
};

use super::{
    super::super::{BufferingSimulateArgsBuilder, SimulationResult},
    info,
};

pub(super) fn dispatch<
    A: Algorithm<O, R, P>,
    O: Scenario<A::MathsCore, A::Rng>,
    R: Reporter,
    P: LocalPartition<R>,
>(
    local_partition: P,

    sample: Sample,
    algorithm_args: A::Arguments,
    scenario: O,
    pause_before: Option<NonNegativeF64>,

    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationResult>
where
    Result<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>:
        anyhow::Context<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>,
{
    let rng: A::Rng = match parse::rng::parse_and_normalise(ron_args, normalised_args)? {
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

    let result = info::dispatch::<A, O, R, P>(
        algorithm_args,
        rng,
        scenario,
        sample,
        pause_before,
        local_partition,
        normalised_args,
    )?;

    match result {
        AlgorithmResult::Done { time, steps } => Ok(SimulationResult::Done { time, steps }),
        AlgorithmResult::Paused {
            time,
            steps,
            lineages,
            rng: paused_rng,
            ..
        } => {
            normalised_args.rng(&RngArgs::State(Base32RngState::from(paused_rng)));

            Ok(SimulationResult::Paused {
                time,
                steps,
                lineages,
            })
        },
    }
}

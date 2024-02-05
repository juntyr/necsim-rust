use anyhow::Context;

use rustcoalescence_algorithms::{result::SimulationOutcome, Algorithm};

use necsim_core::{
    cogs::{MathsCore, RngCore},
    reporter::Reporter,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use crate::args::config::sample::{Sample, SampleMode, SampleModeRestart, SampleOrigin};

pub(super) fn simulate<
    'p,
    M: MathsCore,
    G: RngCore<M>,
    A: Algorithm<'p, M, G, O, R, P>,
    O: Scenario<M, G>,
    R: Reporter,
    P: LocalPartition<'p, R>,
>(
    algorithm_args: A::Arguments,
    rng: G,
    scenario: O,
    sample: Sample,
    pause_before: Option<NonNegativeF64>,
    local_partition: &mut P,
) -> anyhow::Result<SimulationOutcome<M, G>> {
    let lineages = match sample.origin {
        SampleOrigin::Habitat => {
            return A::initialise_and_simulate(
                algorithm_args,
                rng,
                scenario,
                OriginPreSampler::all().percentage(sample.percentage),
                pause_before,
                local_partition,
            )
            .context("Failed to perform the fresh simulation.")
        },
        SampleOrigin::List(lineages) => lineages,
        SampleOrigin::Bincode(loader) => loader.into_lineages(),
    };

    match sample.mode {
        SampleMode::Genesis => A::initialise_and_simulate(
            algorithm_args,
            rng,
            scenario,
            OriginPreSampler::all().percentage(sample.percentage),
            pause_before,
            local_partition,
        )
        .context("Failed to perform the fresh simulation."),
        SampleMode::Resume => A::resume_and_simulate(
            algorithm_args,
            rng,
            scenario,
            OriginPreSampler::all().percentage(sample.percentage),
            lineages.into_iter(),
            None,
            pause_before,
            local_partition,
        )
        .context("Failed to perform the resuming simulation."),
        SampleMode::FixUp(strategy) => A::fixup_for_restart(
            algorithm_args,
            rng,
            scenario,
            OriginPreSampler::all().percentage(sample.percentage),
            lineages.into_iter(),
            PositiveF64::new(pause_before.unwrap().get()).unwrap(),
            strategy,
            local_partition,
        )
        .context("Failed to fix-up the restarting simulation."),
        SampleMode::Restart(SampleModeRestart { after }) => A::resume_and_simulate(
            algorithm_args,
            rng,
            scenario,
            OriginPreSampler::all().percentage(sample.percentage),
            lineages.into_iter(),
            Some(after),
            pause_before,
            local_partition,
        )
        .context("Failed to perform the restarting simulation."),
    }
}

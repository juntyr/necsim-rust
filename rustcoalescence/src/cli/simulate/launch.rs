use anyhow::{Context, Result};

use rustcoalescence_algorithms::{Algorithm, AlgorithmResult};

use necsim_core::reporter::{boolean::Boolean, Reporter};
use necsim_core_bond::NonNegativeF64;
use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use crate::args::{parse::ron_config, Sample};

use super::BufferingSimulateArgsBuilder;

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn launch<
    A: Algorithm<O, R, P>,
    O: Scenario<A::MathsCore, A::Rng>,
    R: Reporter,
    P: LocalPartition<R>,
>(
    algorithm_args: A::Arguments,
    rng: A::Rng,
    scenario: O,
    sample: Sample,
    pause_before: Option<NonNegativeF64>,
    mut local_partition: P,

    normalised_args: &BufferingSimulateArgsBuilder,
) -> anyhow::Result<AlgorithmResult<A::MathsCore, A::Rng>>
where
    Result<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>:
        anyhow::Context<AlgorithmResult<A::MathsCore, A::Rng>, A::Error>,
{
    let config_str = normalised_args
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|simulate_args| {
            ron::ser::to_string_pretty(&simulate_args, ron_config()).map_err(anyhow::Error::new)
        })
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

    if <P::Reporter as Reporter>::ReportSpeciation::VALUE {
        if P::IsLive::VALUE {
            info!("The simulation will report speciation events live.");
        } else {
            info!("The simulation will record speciation events.");
        }
    }

    if <P::Reporter as Reporter>::ReportDispersal::VALUE {
        if P::IsLive::VALUE {
            info!("The simulation will report dispersal events live.");
        } else {
            info!("The simulation will record dispersal events.");
            warn!("Recording dispersal events can be very space-consuming.");
        }
    }

    if <P::Reporter as Reporter>::ReportProgress::VALUE {
        info!("The simulation will report progress events live.");
    }

    if !<P::Reporter as Reporter>::ReportSpeciation::VALUE
        && !<P::Reporter as Reporter>::ReportDispersal::VALUE
        && !<P::Reporter as Reporter>::ReportProgress::VALUE
    {
        warn!("The simulation will report no events.");
    }

    let result = A::initialise_and_simulate(
        algorithm_args,
        rng,
        scenario,
        OriginPreSampler::all().percentage(sample.percentage),
        pause_before,
        &mut local_partition,
    )
    .context("Failed to perform the simulation.")?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n");
        println!(
            "{:=^80}",
            if matches!(result, AlgorithmResult::Done { .. }) {
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
            if matches!(result, AlgorithmResult::Done { .. }) {
                " Reporter Summary "
            } else {
                " Partial Reporter Summary "
            }
        );
        println!();
    }

    Ok(result)
}

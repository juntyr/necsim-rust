use anyhow::{Context, Result};

use rustcoalescence_algorithms::{result::SimulationOutcome, Algorithm};

use necsim_core::{
    cogs::MathsCore,
    reporter::{boolean::Boolean, Reporter},
};
use necsim_core_bond::NonNegativeF64;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use crate::args::{
    config::sample::{Sample, SampleMode, SampleModeRestart},
    utils::parse::try_print,
};

use super::{super::super::BufferingSimulateArgsBuilder, launch};

#[allow(dead_code)]
#[allow(clippy::needless_pass_by_value)]
pub(super) fn dispatch<
    'p,
    M: MathsCore,
    A: Algorithm<'p, M, O, R, P>,
    O: Scenario<M, A::Rng>,
    R: Reporter,
    P: LocalPartition<'p, R>,
>(
    algorithm_args: A::Arguments,
    rng: A::Rng,
    scenario: O,
    sample: Sample,
    pause_before: Option<NonNegativeF64>,
    mut local_partition: P,

    normalised_args: &BufferingSimulateArgsBuilder,
) -> anyhow::Result<SimulationOutcome<M, A::Rng>>
where
    Result<SimulationOutcome<M, A::Rng>, A::Error>:
        anyhow::Context<SimulationOutcome<M, A::Rng>, A::Error>,
{
    let config_str = normalised_args
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|simulate_args| try_print(&simulate_args))
        .context("Failed to normalise the simulation config.")?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n{:=^80}\n", " Simulation Configuration ");
        println!("{}", config_str.trim_start_matches("Simulate"));
        println!("\n{:=^80}\n", " Simulation Configuration ");
    }

    let mut resume_pause = String::from("The simulation will ");
    match sample.mode {
        SampleMode::Genesis => resume_pause.push_str("start fresh"),
        SampleMode::Resume => resume_pause.push_str("resume"),
        SampleMode::FixUp(_) => resume_pause.push_str("fix-up for a restart"),
        SampleMode::Restart(SampleModeRestart { after, .. }) => {
            resume_pause.push_str(&format!("restart after {}", after));
        },
    }
    match pause_before {
        None => resume_pause.push('.'),
        Some(before) => resume_pause.push_str(&format!(" and pause before {}.", before)),
    }
    info!("{}", resume_pause);

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

    let result = launch::simulate::<M, A, O, R, P>(
        algorithm_args,
        rng,
        scenario,
        sample,
        pause_before,
        &mut local_partition,
    )?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n");
        println!(
            "{:=^80}",
            if matches!(result, SimulationOutcome::Done { .. }) {
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
            if matches!(result, SimulationOutcome::Done { .. }) {
                " Reporter Summary "
            } else {
                " Partial Reporter Summary "
            }
        );
        println!();
    }

    Ok(result)
}

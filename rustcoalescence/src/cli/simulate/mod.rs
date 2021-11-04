use anyhow::Context;
use log::LevelFilter;
use serde::Serialize;

use necsim_core_bond::ClosedUnitF64;
use necsim_partitioning_core::Partitioning as _;
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::{
    args::{
        parse::ron_config,
        ser::{BufferingSerialize, BufferingSerializer},
        CommandArgs, Partitioning, Pause, SimulateArgs,
    },
    reporter::DynamicReporterContext,
};

#[macro_use]
mod r#impl;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod dispatch;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger(simulate_args: CommandArgs) -> anyhow::Result<()> {
    log::set_max_level(LevelFilter::Info);

    let simulate_args = SimulateArgs::try_parse(simulate_args)?;

    let partial_resume_args = PartialResumeArgs::from(&simulate_args)?;

    let config_str = ron::ser::to_string_pretty(&simulate_args, ron_config())
        .context("Failed to normalise simulate subcommand arguments.")?;

    let post_validation = move || {
        if log::log_enabled!(log::Level::Info) {
            println!("\n{:=^80}\n", " Simulation Configuration ");
            println!("{}", config_str.trim_start_matches("Simulate"));
            println!("\n{:=^80}\n", " Simulation Configuration ");
        }
    };

    let event_log_directory = simulate_args
        .event_log
        .as_ref()
        .map(|event_log| format!("{:?}", event_log));
    let pre_launch = move || {
        if let Some(event_log_directory) = event_log_directory {
            info!(
                "The simulation will log its events to {}.",
                event_log_directory
            );
            warn!("Therefore, only progress will be reported live.");
        } else {
            info!("The simulation will report events live.");
        }
    };

    match_any_reporter_plugin_vec!(simulate_args.reporters => |reporter| {
        use necsim_partitioning_monolithic::MonolithicLocalPartition;
        #[cfg(feature = "necsim-partitioning-mpi")]
        use necsim_partitioning_mpi::MpiLocalPartition;

        // Initialise the local partition and the simulation
        match simulate_args.partitioning {
            Partitioning::Monolithic(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), simulate_args.event_log
            ).with_context(|| "Failed to initialise the local monolithic partition.")? {
                MonolithicLocalPartition::Live(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
                MonolithicLocalPartition::Recorded(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
            },
            #[cfg(feature = "necsim-partitioning-mpi")]
            Partitioning::Mpi(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), simulate_args.event_log
            ).with_context(|| "Failed to initialise the local MPI partition.")? {
                MpiLocalPartition::Root(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
                MpiLocalPartition::Parallel(partition) => dispatch::simulate_with_logger(
                    partition, simulate_args.common, simulate_args.scenario,
                    simulate_args.algorithm, simulate_args.pause,
                    post_validation, pre_launch,
                ),
            },
        }
    })?;

    let resume_args = ResumeArgs::from(partial_resume_args);

    let resume_str = ron::ser::to_string_pretty(&resume_args, ron_config())
        .context("Failed to generate config to resume the simulation.")?;

    if log::log_enabled!(log::Level::Info) {
        println!("\n{:=^80}\n", " Simulation Configuration ");
        println!("{}", resume_str.trim_start_matches("Simulate"));
        println!("\n{:=^80}\n", " Simulation Configuration ");
    }

    Ok(())
}

#[cfg(not(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
)))]
mod dispatch {
    use necsim_core::reporter::Reporter;
    use necsim_partitioning_core::LocalPartition;

    use crate::args::{
        Algorithm as AlgorithmArgs, CommonArgs, Pause as PauseArgs, Scenario as ScenarioArgs,
    };

    #[allow(clippy::boxed_local, clippy::needless_pass_by_value)]
    pub fn simulate_with_logger<R: Reporter, P: LocalPartition<R>, V: FnOnce(), L: FnOnce()>(
        _local_partition: Box<P>,
        _common_args: CommonArgs,
        _scenario: ScenarioArgs,
        _algorithm: AlgorithmArgs,
        _pause: Option<PauseArgs>,
        _post_validation: V,
        _pre_launch: L,
    ) -> anyhow::Result<()> {
        anyhow::bail!("rustcoalescence must be compiled to support at least one algorithm.")
    }
}

struct PartialResumeArgs {
    speciation: BufferingSerialize,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
}

impl PartialResumeArgs {
    fn from(args: &SimulateArgs) -> anyhow::Result<Self> {
        (|| -> anyhow::Result<Self> {
            Ok(Self {
                speciation: args
                    .common
                    .speciation_probability_per_generation
                    .serialize(BufferingSerializer)?,
                scenario: args.scenario.serialize(BufferingSerializer)?,
                algorithm: args.algorithm.serialize(BufferingSerializer)?,
                partitioning: args.partitioning.serialize(BufferingSerializer)?,
                log: args.event_log.serialize(BufferingSerializer)?,
                reporters: args.reporters.serialize(BufferingSerializer)?,
            })
        })()
        .context("Failed to generate simulation resume config.")
    }
}

#[derive(Serialize)]
#[serde(rename = "Simulate")]
struct ResumeArgs {
    speciation: BufferingSerialize,
    sample: ClosedUnitF64,
    // rng: Rng,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
    pause: Option<Pause>,
}

impl ResumeArgs {
    fn from(partial: PartialResumeArgs) -> Self {
        Self {
            speciation: partial.speciation,
            sample: ClosedUnitF64::one(),
            // rng: Rng,
            scenario: partial.scenario,
            algorithm: partial.algorithm,
            partitioning: partial.partitioning,
            log: partial.log,
            reporters: partial.reporters,
            pause: None,
        }
    }
}

use anyhow::{Context, Result};
use log::LevelFilter;

use necsim_partitioning_core::Partitioning;
use necsim_partitioning_mpi::{MpiLocalPartition, MpiPartitioning};
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::{
    args::{CommandArgs, SimulateArgs},
    reporter::DynamicReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_mpi(simulate_args: CommandArgs) -> Result<()> {
    // Initialise the simulation partitioning
    let partitioning =
        MpiPartitioning::initialise().with_context(|| "Failed to initialise MPI.")?;

    // Only log to stdout/stderr if the partition is the root partition
    log::set_max_level(if partitioning.is_root() {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    });

    let simulate_args = SimulateArgs::try_parse(simulate_args, &partitioning)?;

    let simulate_args_info = format!("{:#?}", simulate_args);
    let post_validation = move || {
        info!("Parsed simulation arguments:\n{}", simulate_args_info);
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
        // Initialise the local partition and the simulation
        match partitioning
            .into_local_partition(
                DynamicReporterContext::new(reporter),
                simulate_args.event_log,
            )
            .with_context(|| "Failed to initialise the local MPI partition.")?
        {
            MpiLocalPartition::LiveMonolithic(partition) => super::simulate_with_logger(
                partition, simulate_args.common, simulate_args.scenario,
                simulate_args.algorithm, post_validation, pre_launch,
            ),
            MpiLocalPartition::RecordedMonolithic(partition) => super::simulate_with_logger(
                partition, simulate_args.common, simulate_args.scenario,
                simulate_args.algorithm, post_validation, pre_launch,
            ),
            MpiLocalPartition::Root(partition) => super::simulate_with_logger(
                partition, simulate_args.common, simulate_args.scenario,
                simulate_args.algorithm, post_validation, pre_launch,
            ),
            MpiLocalPartition::Parallel(partition) => super::simulate_with_logger(
                partition, simulate_args.common, simulate_args.scenario,
                simulate_args.algorithm, post_validation, pre_launch,
            ),
        }
    })
}

use anyhow::{Context, Result};
use log::LevelFilter;

use necsim_impls_mpi::{MpiLocalPartition, MpiPartitioning};
use necsim_impls_no_std::partitioning::Partitioning;
use necsim_plugins_core::match_any_reporter_plugin_vec;

use crate::{
    args::{SimulateArgs, SimulateCommandArgs},
    reporter::DynamicReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_mpi(simulate_args: SimulateCommandArgs) -> Result<()> {
    // Initialise the simulation partitioning
    let partitioning =
        MpiPartitioning::initialise().with_context(|| "Failed to initialise MPI.")?;

    // Only log to stderr if the partition is the root partition
    log::set_max_level(if partitioning.is_root() {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    });

    let simulate_args = SimulateArgs::try_parse(simulate_args, &partitioning)?;
    info!("Parsed simulation arguments:\n{:#?}", simulate_args);

    if let Some(event_log) = &simulate_args.event_log {
        info!(
            "The simulation will log its events to {:?}.",
            event_log.directory()
        );
        warn!("Therefore, only progress will be reported live.");
    }

    match_any_reporter_plugin_vec!(simulate_args.reporters => |reporter| {
        // Initialise the local partition and the simulation
        match partitioning
            .into_local_partition(
                DynamicReporterContext::new(reporter),
                simulate_args.event_log,
            )
            .with_context(|| "Failed to initialise the local MPI partition.")?
        {
            MpiLocalPartition::LiveMonolithic(partition) => {
                super::simulate_with_logger(partition, simulate_args.common, simulate_args.scenario)
            },
            MpiLocalPartition::RecordedMonolithic(partition) => {
                super::simulate_with_logger(partition, simulate_args.common, simulate_args.scenario)
            },
            MpiLocalPartition::Root(partition) => {
                super::simulate_with_logger(partition, simulate_args.common, simulate_args.scenario)
            },
            MpiLocalPartition::Parallel(partition) => {
                super::simulate_with_logger(partition, simulate_args.common, simulate_args.scenario)
            },
        }
    })
}

use std::convert::TryInto;

use anyhow::{Context, Result};
use log::LevelFilter;

use necsim_impls_mpi::{MpiLocalPartition, MpiPartitioning};
use necsim_impls_no_std::partitioning::Partitioning;
use necsim_impls_std::event_log::recorder::EventLogRecorder;

use crate::{
    args::{SimulateArgs, SimulateCommandArgs},
    reporter::RustcoalescenceReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_mpi(simulate_args: SimulateCommandArgs) -> Result<()> {
    // Initialise the simulation partitioning
    let partitioning =
        MpiPartitioning::initialise().with_context(|| "Failed to initialise MPI.")?;

    // TODO: Move check to algorithm
    // #[cfg(feature = "necsim-independent")]
    // if let crate::args::Algorithm::Independent(necsim_independent::
    // IndependentArguments {     partition_mode:
    // necsim_independent::PartitionMode::IsolatedIndividuals(         _rank,
    // partitions     ),
    //     ..
    // }) = simulate_args.common_args().algorithm()
    // {
    //     if partitions.get() > 1 && !partitioning.is_monolithic() {
    //         anyhow::bail!("MPI partitioning is incompatible with isolated
    // partitions.");     }
    // }

    // Only log to stderr if the partition is the root partition
    log::set_max_level(if partitioning.is_root() {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    });

    let simulate_args: SimulateArgs = simulate_args.try_into()?;
    info!("Parsed simulation arguments:\n{:#?}", simulate_args);

    let is_monolithic = partitioning.is_monolithic();

    let event_log = match simulate_args.event_log {
        Some(mut event_log_path) => {
            event_log_path.push(partitioning.get_rank().to_string());

            Some(EventLogRecorder::try_new(&event_log_path)?)
        },
        None => None,
    };

    // Initialise the local partition and the simulation
    match partitioning
        .into_local_partition(
            RustcoalescenceReporterContext::new(is_monolithic),
            event_log,
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
}

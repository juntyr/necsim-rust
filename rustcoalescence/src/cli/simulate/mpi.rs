use anyhow::{Context, Result};
use log::LevelFilter;

use necsim_impls_mpi::{MpiLocalPartition, MpiPartitioning};
use necsim_impls_no_std::partitioning::Partitioning;

use crate::{
    args::{RustcoalescenceArgs, SimulateArgs},
    reporter::RustcoalescenceReporterContext,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger_mpi(
    args: &RustcoalescenceArgs,
    simulate_args: &SimulateArgs,
) -> Result<()> {
    // Initialise the simulation partitioning
    let partitioning =
        MpiPartitioning::initialise(simulate_args.common_args().event_log().as_deref())
            .with_context(|| "Failed to initialise MPI.")?;

    #[cfg(feature = "necsim-independent")]
    if let crate::args::Algorithm::Independent(necsim_independent::IndependentArguments {
        partition_mode: necsim_independent::PartitionMode::IsolatedIndividuals(_rank, partitions),
        ..
    }) = simulate_args.common_args().algorithm()
    {
        if partitions.get() > 1 && !partitioning.is_monolithic() {
            anyhow::bail!("MPI partitioning is incompatible with isolated partitions.");
        }
    }

    // Only log to stderr if the partition is the root partition
    log::set_max_level(if partitioning.is_root() {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    });

    info!("Parsed arguments:\n{:#?}", args);

    let is_monolithic = partitioning.is_monolithic();

    // Initialise the local partition and the simulation
    match partitioning.into_local_partition(RustcoalescenceReporterContext::new(is_monolithic)) {
        MpiLocalPartition::LiveMonolithic(partition) => {
            super::simulate_with_logger(partition, simulate_args)
        },
        MpiLocalPartition::RecordedMonolithic(partition) => {
            super::simulate_with_logger(partition, simulate_args)
        },
        MpiLocalPartition::Root(partition) => super::simulate_with_logger(partition, simulate_args),
        MpiLocalPartition::Parallel(partition) => {
            super::simulate_with_logger(partition, simulate_args)
        },
    }
}

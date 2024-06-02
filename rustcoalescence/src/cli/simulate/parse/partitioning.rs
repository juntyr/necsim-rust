use serde::Deserialize;

use crate::args::{config::partitioning::Partitioning, utils::parse::try_parse};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<Partitioning> {
    let SimulateArgsPartitioningOnly { partitioning } = try_parse("simulate", ron_args)?;

    normalised_args.partitioning(&partitioning);

    #[cfg(feature = "mpi-partitioning")]
    if !matches!(&partitioning, Partitioning::Mpi(_)) {
        match necsim_partitioning_mpi::MpiPartitioning::initialise() {
            Ok(_) | Err(necsim_partitioning_mpi::MpiPartitioningError::AlreadyInitialised) => {
                anyhow::bail!("MPI should not be used together with a non-MPI partitioning")
            },
            Err(necsim_partitioning_mpi::MpiPartitioningError::NoParallelism) => (),
        }
    }

    Ok(partitioning)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsPartitioningOnly {
    #[serde(default)]
    partitioning: Partitioning,
}

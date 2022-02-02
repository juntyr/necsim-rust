use serde::Deserialize;

use crate::args::{config::partitioning::Partitioning, utils::parse::try_parse};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<Partitioning> {
    let SimulateArgsPartitioningOnly { partitioning } = try_parse("simulate", ron_args)?;

    normalised_args.partitioning(&partitioning);

    Ok(partitioning)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsPartitioningOnly {
    #[serde(default)]
    partitioning: Partitioning,
}

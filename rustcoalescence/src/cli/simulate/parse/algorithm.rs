use necsim_partitioning_core::partition::Partition;

use crate::args::{parse::try_parse_state, Algorithm, Partitioning};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
) -> anyhow::Result<Algorithm> {
    let SimulateArgsAlgorithmOnly { algorithm } =
        try_parse_state("simulate", ron_args, &mut partitioning.get_partition())?;

    normalised_args.algorithm(&algorithm);

    Ok(algorithm)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "Partition")]
#[serde(rename = "Simulate")]
struct SimulateArgsAlgorithmOnly {
    #[serde(deserialize_state)]
    algorithm: Algorithm,
}

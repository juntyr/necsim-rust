use necsim_partitioning_core::partition::PartitionSize;

use crate::args::{
    config::{algorithm::Algorithm, partitioning::Partitioning},
    utils::parse::try_parse_state,
};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
) -> anyhow::Result<Algorithm> {
    let SimulateArgsAlgorithmOnly { algorithm } =
        try_parse_state("simulate", ron_args, &mut partitioning.get_size())?;

    normalised_args.algorithm(&algorithm);

    Ok(algorithm)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "PartitionSize")]
#[serde(rename = "Simulate")]
struct SimulateArgsAlgorithmOnly {
    #[serde(deserialize_state)]
    algorithm: Algorithm,
}

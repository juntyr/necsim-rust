use necsim_partitioning_core::partition::PartitionSize;

use crate::args::{
    config::{partitioning::Partitioning, pause::Pause},
    utils::parse::try_parse_state,
};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
) -> anyhow::Result<Option<Pause>> {
    let SimulateArgsPauseOnly { pause } =
        try_parse_state("simulate", ron_args, &mut partitioning.get_size())?;

    normalised_args.pause(&pause);

    Ok(pause)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "PartitionSize")]
#[serde(rename = "Simulate")]
struct SimulateArgsPauseOnly {
    #[serde(default)]
    #[serde(deserialize_state)]
    pause: Option<Pause>,
}

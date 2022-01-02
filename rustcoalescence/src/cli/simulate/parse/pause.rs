use necsim_partitioning_core::partition::Partition;

use crate::args::{parse::try_parse_state, Partitioning, Pause, Sample};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise<'a>(
    ron_args: &'a str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
    sample: &'a Sample,
) -> anyhow::Result<Option<Pause>> {
    let SimulateArgsPauseOnly { pause } = try_parse_state(
        "simulate",
        ron_args,
        &mut (partitioning.get_partition(), sample),
    )?;

    normalised_args.pause(&pause);

    Ok(pause)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "(Partition, &'de Sample)")]
#[serde(rename = "Simulate")]
struct SimulateArgsPauseOnly {
    #[serde(default)]
    #[serde(deserialize_state)]
    pause: Option<Pause>,
}

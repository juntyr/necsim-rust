use necsim_core_bond::Partition;

use crate::args::{parse::try_parse_state, Partitioning, Pause, Sample};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partitioning: &Partitioning,
    _sample: &Sample,
) -> anyhow::Result<Option<Pause>> {
    let SimulateArgsPauseOnly { pause } =
        try_parse_state("simulate", ron_args, &mut partitioning.get_partition())?;

    // TODO: Validate that Pause::List is only allowed if sample.origin::List

    normalised_args.pause(&pause);

    Ok(pause)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "Partition")]
#[serde(rename = "Simulate")]
struct SimulateArgsPauseOnly {
    #[serde(default)]
    #[serde(deserialize_state)]
    pause: Option<Pause>,
}

use crate::args::{parse::try_parse_state, Pause, Sample};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise<'a>(
    ron_args: &'a str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    mut pause: &'a Option<Pause>,
) -> anyhow::Result<Sample> {
    let SimulateArgsSampleOnly { sample } = try_parse_state("simulate", ron_args, &mut pause)?;

    normalised_args.sample(&sample);

    Ok(sample)
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "&'de Option<Pause>")]
#[serde(rename = "Simulate")]
struct SimulateArgsSampleOnly {
    #[serde(default)]
    #[serde(deserialize_state)]
    sample: Sample,
}

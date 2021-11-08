use serde::Deserialize;

use crate::args::{parse::try_parse, Sample};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<Sample> {
    let SimulateArgsSampleOnly { sample } = try_parse("simulate", ron_args)?;

    normalised_args.sample(&sample);

    Ok(sample)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsSampleOnly {
    #[serde(default)]
    sample: Sample,
}

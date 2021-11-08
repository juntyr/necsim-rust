use serde::Deserialize;

use crate::args::{parse::try_parse, Scenario};

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<Scenario> {
    let SimulateArgsScenarioOnly { scenario } = try_parse("simulate", ron_args)?;

    normalised_args.scenario(&scenario);

    Ok(scenario)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsScenarioOnly {
    scenario: Scenario,
}

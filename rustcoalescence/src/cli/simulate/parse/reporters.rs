use serde::Deserialize;

use necsim_plugins_core::import::AnyReporterPluginVec;

use crate::args::parse::try_parse;

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<AnyReporterPluginVec> {
    let SimulateArgsReportersOnly { reporters } = try_parse("simulate", ron_args)?;

    normalised_args.reporters(&reporters);

    Ok(reporters)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsReportersOnly {
    reporters: AnyReporterPluginVec,
}

use serde::Deserialize;

use necsim_core_bond::OpenClosedUnitF64 as PositiveUnitF64;

use crate::args::parse::try_parse;

use super::super::BufferingSimulateArgsBuilder;

pub(in super::super) fn parse_and_normalise(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<PositiveUnitF64> {
    let SimulateArgsSpeciationOnly {
        speciation_probability_per_generation,
    } = try_parse("simulate", ron_args)?;

    normalised_args.speciation(&speciation_probability_per_generation);

    Ok(speciation_probability_per_generation)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsSpeciationOnly {
    #[serde(alias = "speciation")]
    speciation_probability_per_generation: PositiveUnitF64,
}

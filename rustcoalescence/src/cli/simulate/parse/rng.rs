use serde::Deserialize;

use necsim_core::cogs::{MathsCore, RngCore};

use crate::args::{config::rng::Rng, utils::parse::try_parse};

use super::super::BufferingSimulateArgsBuilder;

#[allow(dead_code)]
pub(in super::super) fn parse_and_normalise<M: MathsCore, G: RngCore<M>>(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
) -> anyhow::Result<Rng<M, G>> {
    let SimulateArgsRngOnly { rng } = try_parse("simulate", ron_args)?;

    normalised_args.rng(&rng);

    Ok(rng)
}

#[derive(Deserialize)]
#[serde(bound = "")]
#[serde(rename = "Simulate")]
struct SimulateArgsRngOnly<M: MathsCore, G: RngCore<M>> {
    #[serde(alias = "randomness")]
    rng: Rng<M, G>,
}

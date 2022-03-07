use necsim_core::cogs::{MathsCore, RngCore};
use necsim_partitioning_core::partition::Partition;

use crate::args::{config::rng::Rng, utils::parse::try_parse_state};

use super::super::BufferingSimulateArgsBuilder;

#[allow(dead_code)]
pub(in super::super) fn parse_and_normalise<M: MathsCore, G: RngCore<M>>(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partition: &mut Partition,
) -> anyhow::Result<Rng<M, G>> {
    let SimulateArgsRngOnly { rng } = try_parse_state("simulate", ron_args, partition)?;

    normalised_args.rng(&rng);

    Ok(rng)
}

#[derive(DeserializeState)]
#[serde(bound = "")]
#[serde(rename = "Simulate")]
#[serde(deserialize_state = "Partition")]
struct SimulateArgsRngOnly<M: MathsCore, G: RngCore<M>> {
    #[serde(alias = "randomness")]
    #[serde(deserialize_state)]
    rng: Rng<M, G>,
}

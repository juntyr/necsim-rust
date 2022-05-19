use necsim_core::cogs::RngCore;
use necsim_partitioning_core::partition::Partition;

use crate::args::{config::rng::RngConfig, utils::parse::try_parse_state};

use super::super::BufferingSimulateArgsBuilder;

#[allow(dead_code)]
pub(in super::super) fn parse_and_normalise<G: RngCore>(
    ron_args: &str,
    normalised_args: &mut BufferingSimulateArgsBuilder,
    partition: &mut Partition,
) -> anyhow::Result<RngConfig<G>> {
    let SimulateArgsRngOnly { rng } = try_parse_state("simulate", ron_args, partition)?;

    normalised_args.rng(&rng);

    Ok(rng)
}

#[derive(DeserializeState)]
#[serde(rename = "Simulate")]
#[serde(deserialize_state = "Partition")]
struct SimulateArgsRngOnly<G: RngCore> {
    #[serde(alias = "randomness")]
    #[serde(deserialize_state)]
    rng: RngConfig<G>,
}

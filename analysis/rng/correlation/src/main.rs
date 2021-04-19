#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]

#[macro_use]
extern crate contracts;

use structopt::StructOpt;

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyhash::WyHash;

mod simulation;
use simulation::CorrelationSimulationRng;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "necsim 4 x randomness generator",
    about = "Generates 4 adjacent streams of random bytes to stdout."
)]
struct Options {
    #[structopt(long)]
    seed: u64,
    #[structopt(long)]
    limit: u128,
}

fn main() {
    let options = Options::from_args();

    let mut rng = CorrelationSimulationRng::<WyHash>::seed_from_u64(options.seed);

    for _ in 0..options.limit {
        println!(
            "{},{},{},{}",
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64()
        )
    }
}

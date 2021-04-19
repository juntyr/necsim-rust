#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]
#![allow(incomplete_features)]
#![feature(const_generics)]

#[macro_use]
extern crate contracts;

use structopt::StructOpt;

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyhash::WyHash;

mod simulation;
use simulation::CorrelationSimulationRng;

#[derive(Debug, StructOpt)]
enum DispersalMode {
    NoDispersal,
    HighDispersal,
}

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
    #[structopt(subcommand)]
    mode: DispersalMode,
}

fn main() {
    let options = Options::from_args();

    match options.mode {
        DispersalMode::NoDispersal => sample_random_streams(
            CorrelationSimulationRng::<WyHash, 0.0>::seed_from_u64(options.seed),
            options.limit,
        ),
        DispersalMode::HighDispersal => sample_random_streams(
            CorrelationSimulationRng::<WyHash, 100.0>::seed_from_u64(options.seed),
            options.limit,
        ),
    }
}

fn sample_random_streams<R: RngCore>(mut rng: R, limit: u128) {
    for _ in 0..limit {
        println!(
            "{},{},{},{}",
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64()
        )
    }
}

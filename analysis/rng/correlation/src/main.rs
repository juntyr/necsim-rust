#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]
#![feature(control_flow_enum)]
#![allow(incomplete_features)]
#![feature(adt_const_params)]

#[macro_use]
extern crate contracts;

use structopt::StructOpt;

use necsim_core::cogs::{MathsCore, RngCore, SeedableRng};
use necsim_core_maths::IntrinsicsMathsCore;
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
            CorrelationSimulationRng::<IntrinsicsMathsCore, WyHash<_>, 0.0>::seed_from_u64(
                options.seed,
            ),
            options.limit,
        ),
        DispersalMode::HighDispersal => sample_random_streams(
            CorrelationSimulationRng::<IntrinsicsMathsCore, WyHash<_>, 100.0>::seed_from_u64(
                options.seed,
            ),
            options.limit,
        ),
    }
}

fn sample_random_streams<M: MathsCore, R: RngCore<M>>(mut rng: R, limit: u128) {
    for _ in 0..limit {
        println!(
            "{},{},{},{}",
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64(),
            rng.sample_u64()
        );
    }
}

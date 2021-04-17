#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use std::io::{self, BufWriter, Write};

use byte_unit::{Byte, ByteError};
use structopt::StructOpt;

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyhash::WyHash;
use necsim_impls_std::cogs::rng::pcg::Pcg;

mod simulation;
use simulation::SimulationRng;

#[derive(Debug, StructOpt)]
enum GeneratorMode {
    Monolithic,
    Independent,
    IndependentSimulation,
    IndependentSimulationNoDispersal,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "necsim randomness generator",
    about = "Generates a stream of random bytes to stdout."
)]
struct Options {
    #[structopt(long)]
    seed: u64,
    #[structopt(long, parse(try_from_str = try_parse_length))]
    limit: Option<u128>,
    #[structopt(subcommand)]
    mode: GeneratorMode,
}

fn try_parse_length(s: &str) -> Result<u128, ByteError> {
    s.parse::<Byte>().map(|byte| byte.get_bytes())
}

fn main() -> io::Result<()> {
    let options = Options::from_args();

    let stdout = BufWriter::with_capacity(4096, io::stdout());

    match (options.limit, options.mode) {
        (None, GeneratorMode::Monolithic) => {
            produce_unlimited_randomness(stdout, Pcg::seed_from_u64(options.seed))
        },
        (None, GeneratorMode::Independent) => {
            produce_unlimited_randomness(stdout, WyHash::seed_from_u64(options.seed))
        },
        (None, GeneratorMode::IndependentSimulation) => produce_unlimited_randomness(
            stdout,
            SimulationRng::<WyHash, 100>::seed_from_u64(options.seed),
        ),
        (None, GeneratorMode::IndependentSimulationNoDispersal) => produce_unlimited_randomness(
            stdout,
            SimulationRng::<WyHash, 1>::seed_from_u64(options.seed),
        ),
        (Some(limit), GeneratorMode::Monolithic) => {
            produce_limited_randomness(stdout, Pcg::seed_from_u64(options.seed), limit)
        },
        (Some(limit), GeneratorMode::Independent) => {
            produce_limited_randomness(stdout, WyHash::seed_from_u64(options.seed), limit)
        },
        (Some(limit), GeneratorMode::IndependentSimulation) => produce_limited_randomness(
            stdout,
            SimulationRng::<WyHash, 100>::seed_from_u64(options.seed),
            limit,
        ),
        (Some(limit), GeneratorMode::IndependentSimulationNoDispersal) => {
            produce_limited_randomness(
                stdout,
                SimulationRng::<WyHash, 1>::seed_from_u64(options.seed),
                limit,
            )
        },
    }
}

fn produce_limited_randomness<W: Write, R: RngCore>(
    mut writer: W,
    mut rng: R,
    limit: u128,
) -> io::Result<()> {
    for _ in 0..(limit / 8) {
        writer.write_all(&rng.sample_u64().to_le_bytes())?
    }

    writer.write_all(&rng.sample_u64().to_le_bytes()[..((limit % 8) as usize)])
}

fn produce_unlimited_randomness<W: Write, R: RngCore>(mut writer: W, mut rng: R) -> io::Result<()> {
    loop {
        writer.write_all(&rng.sample_u64().to_le_bytes())?
    }
}

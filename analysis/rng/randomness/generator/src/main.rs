#![deny(clippy::pedantic)]

use std::io::{self, BufWriter, Write};

use byte_unit::{Byte, ByteError};
use structopt::StructOpt;

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyrand::WyRand;

#[derive(Debug, StructOpt)]
enum GeneratorMode {
    Monolithic,
    Independent,
    IndependentSimulation,
    IndependentSimulationFixedLocation,
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

    println!("{:?}", options);

    let stdout = BufWriter::with_capacity(4096, io::stdout());

    let rng = WyRand::seed_from_u64(options.seed);

    if let Some(limit) = options.limit {
        produce_limited_randomness(stdout, rng, limit)
    } else {
        produce_unlimited_randomness(stdout, rng)
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

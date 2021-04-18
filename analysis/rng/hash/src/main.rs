#![deny(clippy::pedantic)]

use std::io::{self, BufWriter, Write};

use rand::{rngs::StdRng, RngCore, SeedableRng};
use structopt::StructOpt;

use necsim_core::cogs::{PrimeableRng, RngCore as _};
use necsim_impls_no_std::cogs::rng::wyhash::WyHash;

#[derive(Debug, StructOpt)]
enum HashMode {
    Update,
    Prime,
}

#[derive(Debug, StructOpt)]
#[structopt(
    name = "necsim hashing avalanche test",
    about = "Generates a sequence of hashing examples."
)]
struct Options {
    #[structopt(long)]
    seed: u64,
    #[structopt(long)]
    limit: u128,
    #[structopt(subcommand)]
    mode: HashMode,
}

fn main() -> io::Result<()> {
    let options = Options::from_args();

    let mut rng = StdRng::seed_from_u64(options.seed);
    let mut stdout = BufWriter::new(io::stdout());

    match options.mode {
        HashMode::Update => {
            test_update_hash(u64::MIN, &mut stdout)?;
            for _ in 0..options.limit {
                test_update_hash(rng.next_u64(), &mut stdout)?
            }
            test_update_hash(u64::MAX, &mut stdout)?;
        },
        HashMode::Prime => {
            test_prime_hash(u64::MIN, u64::MIN, u64::MIN, &mut stdout)?;
            for _ in 0..options.limit {
                test_prime_hash(rng.next_u64(), rng.next_u64(), rng.next_u64(), &mut stdout)?;
            }
            test_prime_hash(u64::MAX, u64::MAX, u64::MAX, &mut stdout)?;
        },
    }

    Ok(())
}

fn test_update_hash<W: Write>(state: u64, writer: &mut W) -> io::Result<()> {
    let mut rng_origin = WyHash::from_seed(state.to_le_bytes());
    let hash_origin = rng_origin.sample_u64();

    for i in 0..64 {
        let mut rng_flipped = WyHash::from_seed((state ^ (0x1_u64 << i)).to_le_bytes());
        let hash_flipped = rng_flipped.sample_u64();

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    Ok(())
}

fn test_prime_hash<W: Write>(
    seed: u64,
    location_index: u64,
    time_index: u64,
    writer: &mut W,
) -> io::Result<()> {
    let mut rng_origin = WyHash::seed_from_u64(seed);
    rng_origin.prime_with(location_index, time_index);
    let hash_origin = rng_origin.sample_u64();

    for i in 0..64 {
        let mut rng_flipped = WyHash::seed_from_u64(seed ^ (0x1_u64 << i));
        rng_origin.prime_with(location_index, time_index);
        let hash_flipped = rng_flipped.sample_u64();

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    let mut rng_flipped = WyHash::seed_from_u64(seed);

    for i in 0..64 {
        rng_origin.prime_with(location_index ^ (0x1_u64 << i), time_index);
        let hash_flipped = rng_flipped.sample_u64();

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    for i in 0..64 {
        rng_origin.prime_with(location_index, time_index ^ (0x1_u64 << i));
        let hash_flipped = rng_flipped.sample_u64();

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    Ok(())
}

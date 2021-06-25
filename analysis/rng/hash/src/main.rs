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
    #[structopt(long)]
    raw_prime: bool,
    #[structopt(long)]
    raw_output: bool,
    #[structopt(subcommand)]
    mode: HashMode,
}

fn main() -> io::Result<()> {
    let options = Options::from_args();

    let mut rng = StdRng::seed_from_u64(options.seed);
    let mut stdout = BufWriter::new(io::stdout());

    match options.mode {
        HashMode::Update => {
            test_update_hash(u64::MIN, &mut stdout, options.raw_output)?;
            for _ in 0..options.limit {
                test_update_hash(rng.next_u64(), &mut stdout, options.raw_output)?
            }
            test_update_hash(u64::MAX, &mut stdout, options.raw_output)?;
        },
        HashMode::Prime => {
            test_prime_hash(
                u64::MIN,
                u64::MIN,
                u64::MIN,
                &mut stdout,
                options.raw_prime,
                options.raw_output,
            )?;
            for _ in 0..options.limit {
                test_prime_hash(
                    rng.next_u64(),
                    rng.next_u64(),
                    rng.next_u64(),
                    &mut stdout,
                    options.raw_prime,
                    options.raw_output,
                )?;
            }
            test_prime_hash(
                u64::MAX,
                u64::MAX,
                u64::MAX,
                &mut stdout,
                options.raw_prime,
                options.raw_output,
            )?;
        },
    }

    Ok(())
}

fn test_update_hash<W: Write>(state: u64, writer: &mut W, raw_output: bool) -> io::Result<()> {
    let mut rng_origin = WyHash::from_seed(state.to_le_bytes());
    let hash_origin = optional_undiffuse(rng_origin.sample_u64(), raw_output);

    for i in 0..64 {
        let mut rng_flipped = WyHash::from_seed((state ^ (0x1_u64 << i)).to_le_bytes());
        let hash_flipped = optional_undiffuse(rng_flipped.sample_u64(), raw_output);

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    Ok(())
}

fn test_prime_hash<W: Write>(
    seed: u64,
    location_index: u64,
    time_index: u64,
    writer: &mut W,
    raw_prime: bool,
    raw_output: bool,
) -> io::Result<()> {
    let mut rng_origin = WyHash::seed_from_u64(seed);
    rng_origin.prime_with(
        optional_undiffuse(location_index, raw_prime),
        optional_undiffuse(time_index, raw_prime),
    );
    let hash_origin = optional_undiffuse(rng_origin.sample_u64(), raw_output);

    for i in 0..64 {
        let mut rng_flipped = WyHash::seed_from_u64(seed ^ (0x1_u64 << i));
        rng_origin.prime_with(
            optional_undiffuse(location_index, raw_prime),
            optional_undiffuse(time_index, raw_prime),
        );
        let hash_flipped = optional_undiffuse(rng_flipped.sample_u64(), raw_output);

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    let mut rng_flipped = WyHash::seed_from_u64(seed);

    for i in 0..64 {
        rng_origin.prime_with(
            optional_undiffuse(location_index ^ (0x1_u64 << i), raw_prime),
            optional_undiffuse(time_index, raw_prime),
        );
        let hash_flipped = optional_undiffuse(rng_flipped.sample_u64(), raw_output);

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    for i in 0..64 {
        rng_origin.prime_with(
            optional_undiffuse(location_index, raw_prime),
            optional_undiffuse(time_index ^ (0x1_u64 << i), raw_prime),
        );
        let hash_flipped = optional_undiffuse(rng_flipped.sample_u64(), raw_output);

        writeln!(writer, "{}", hash_origin ^ hash_flipped)?
    }

    Ok(())
}

fn optional_undiffuse(x: u64, undiffuse: bool) -> u64 {
    if undiffuse {
        seahash_undiffuse(x)
    } else {
        x
    }
}

#[must_use]
pub const fn seahash_undiffuse(mut x: u64) -> u64 {
    // SeaHash undiffusion function
    // https://docs.rs/seahash/4.1.0/src/seahash/helper.rs.html#94-105

    // 0x2f72b4215a3d8caf is the modular multiplicative inverse of the constant used
    //  in `diffuse`.

    x = x.wrapping_mul(0x2f72_b421_5a3d_8caf);

    let a = x >> 32;
    let b = x >> 60;

    x ^= a >> b;

    x = x.wrapping_mul(0x2f72_b421_5a3d_8caf);

    x
}

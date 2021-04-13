use std::io::{Write, Result};

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::fixedseahash::FixedSeaHash;

fn main() -> Result<()> {
    let mut rng = FixedSeaHash::seed_from_u64(12345678);

    let mut stdout = std::io::stdout();

    loop {
        stdout.write(&rng.sample_u64().to_le_bytes())?;
    }
}

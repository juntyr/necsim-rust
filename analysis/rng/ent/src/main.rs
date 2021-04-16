#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use std::{
    io,
    process::{Command, Stdio},
};

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyhash::WyHash;
use rng::WriteInterceptingReporter;

mod rng;
mod simulate;

const BUFFER_SIZE: usize = 1024;

fn main() -> io::Result<()> {
    let length_arg = std::env::args().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "First parameter must be the length of the random stream to test.",
        )
    })?;
    let length: u128 = byte_unit::Byte::from_str(length_arg.trim())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?
        .get_bytes();

    let rng = WyHash::seed_from_u64(823789433274912);

    let mut command = Command::new("./ent")
        .args(std::env::args_os().skip(2))
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let stdin = command.stdin.take().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Pipe to ent could not be created.",
        )
    })?;

    let rng = WriteInterceptingReporter::new(rng, &stdin, BUFFER_SIZE);

    #[allow(clippy::cast_precision_loss)]
    simulate::simulate(rng, (length as f64).recip());

    Ok(())
}

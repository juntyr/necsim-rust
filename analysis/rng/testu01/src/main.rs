#![deny(clippy::pedantic)]

use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::fixedseahash::FixedSeaHash;

const BUFFER_SIZE: usize = 512;

fn main() -> io::Result<()> {
    let mut rng = FixedSeaHash::seed_from_u64(123_456_789);

    let mut buffer = vec![0_u64; BUFFER_SIZE].into_boxed_slice();

    let mut command = Command::new("./testu01")
        .args(std::env::args_os().skip(1))
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdin = command.stdin.take().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Pipe to testu01 could not be created.",
        )
    })?;

    loop {
        buffer.fill_with(|| rng.sample_u64());

        if let Err(err) = stdin.write_all(unsafe {
            std::slice::from_raw_parts(buffer.as_ptr().cast(), buffer.len() * 8)
        }) {
            if err.kind() == io::ErrorKind::BrokenPipe {
                return Ok(());
            }

            return Err(err);
        }
    }
}

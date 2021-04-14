#![deny(clippy::pedantic)]

use std::{
    io::{self, Write},
    process::{Command, Stdio},
};

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::fixedseahash::FixedSeaHash;

const BUFFER_SIZE: usize = 1024;

fn main() -> io::Result<()> {
    let length_arg = std::env::args().nth(1).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "First parameter must be the length of the random stream to test.",
        )
    })?;
    let mut length: u128 = byte_unit::Byte::from_str(length_arg.trim())
        .map_err(|err| io::Error::new(io::ErrorKind::InvalidInput, err))?
        .get_bytes();

    let mut rng = FixedSeaHash::seed_from_u64(123_456_789);

    let mut buffer = vec![0_u64; BUFFER_SIZE].into_boxed_slice();

    let mut command = Command::new("./ent")
        .args(std::env::args_os().skip(2))
        .stdin(Stdio::piped())
        .stdout(Stdio::inherit())
        .stderr(Stdio::inherit())
        .spawn()?;
    let mut stdin = command.stdin.take().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::BrokenPipe,
            "Pipe to ent could not be created.",
        )
    })?;

    while length > 0 {
        buffer.fill_with(|| rng.sample_u64());

        let byte_slice: &[u8] =
            unsafe { std::slice::from_raw_parts(buffer.as_ptr().cast(), buffer.len() * 8) };

        stdin.write_all(&byte_slice[..(length.min(BUFFER_SIZE as u128) as usize)])?;

        length = length.saturating_sub(BUFFER_SIZE as u128);
    }

    Ok(())
}

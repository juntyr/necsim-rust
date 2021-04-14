#![deny(clippy::pedantic)]
#![deny(clippy::pedantic)]

use std::{path::PathBuf, process::Command};

fn main() -> std::io::Result<()> {
    let mut libpath = PathBuf::from(std::env::var_os("CRATE_OUT_DIR").unwrap());
    libpath.push("libtestu01.a");

    Command::new("make")
        .env("LIB_NECSIM_RNG", &libpath)
        .status()
        .map(|_| ())
}

#![deny(clippy::pedantic)]
#![feature(unwrap_infallible)]

#[macro_use]
extern crate serde_derive_state;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use structopt::StructOpt;

mod args;
mod cli;
mod minimal_logger;
mod reporter;

use args::RustcoalescenceArgs;
use minimal_logger::MinimalLogger;

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() -> Result<()> {
    // Set up the minimal logger to stdout/stderr
    log::set_logger(&MINIMAL_LOGGER)?;

    // Parse and validate all command line arguments
    let args = RustcoalescenceArgs::from_args();

    match args {
        RustcoalescenceArgs::Simulate(simulate_args) => {
            cli::simulate::simulate_with_logger(simulate_args)
                .context("Failed to initialise or perform the simulation.")
        },
        RustcoalescenceArgs::Replay(replay_args) => {
            cli::replay::replay_with_logger(replay_args).context("Failed to replay the simulation.")
        },
    }
}

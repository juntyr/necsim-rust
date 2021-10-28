#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]
#![feature(unwrap_infallible)]
#![feature(box_patterns)]
#![feature(cursor_remaining)]
#![allow(incomplete_features)]
#![feature(specialization)]

#[macro_use]
extern crate serde_derive_state;

#[macro_use]
extern crate log;

use anyhow::{Context, Result};
use log::LevelFilter;
use structopt::StructOpt;

mod args;
mod cli;
mod maps;
mod minimal_logger;
mod reporter;
mod tiff;

use args::RustcoalescenceArgs;
use minimal_logger::MinimalLogger;

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() -> Result<()> {
    // Set up the minimal logger to stdout/stderr
    log::set_logger(&MINIMAL_LOGGER)?;

    // Parse and validate all command line arguments
    let args = RustcoalescenceArgs::from_args();

    let result = match args {
        RustcoalescenceArgs::Simulate(simulate_args) => {
            cli::simulate::simulate_with_logger(simulate_args)
                .context("Failed to initialise or perform the simulation.")
        },
        RustcoalescenceArgs::Replay(replay_args) => {
            cli::replay::replay_with_logger(replay_args).context("Failed to replay the simulation.")
        },
    };

    // Hide non-root error messages
    if log::max_level() == LevelFilter::Off {
        Ok(())
    } else {
        result
    }
}

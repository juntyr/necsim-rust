#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(associated_type_bounds)]

#[macro_use]
extern crate necsim_core;

#[macro_use]
extern crate log;

#[allow(unused_imports)]
use anyhow::{Context, Result};
use log::LevelFilter;
use structopt::StructOpt;

mod args;
mod cli;
mod maps;
mod minimal_logger;
mod reporter;
mod simulation;
mod tiff;

use args::RustcoalescenceArgs;
use minimal_logger::MinimalLogger;
use reporter::RustcoalescenceReporterContext;

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() -> Result<()> {
    // Parse and validate all command line arguments
    let args = RustcoalescenceArgs::from_args();

    log::set_logger(&MINIMAL_LOGGER)?;

    match args {
        RustcoalescenceArgs::Simulate(ref simulate_args) => {
            #[cfg(feature = "necsim-mpi")]
            {
                cli::simulate::mpi::simulate_with_logger_mpi(&args, simulate_args)
            }
            #[cfg(not(feature = "necsim-mpi"))]
            {
                cli::simulate::monolithic::simulate_with_logger_monolithic(&args, simulate_args)
            }
        }
        .context("Failed to initialise or perform the simulation."),

        RustcoalescenceArgs::Replay(ref replay_args) => {
            // Always log to stderr (replay is run without partitioning)
            log::set_max_level(LevelFilter::Info);

            info!("Parsed arguments:\n{:#?}", args);

            cli::replay::replay_with_logger(replay_args, RustcoalescenceReporterContext::new(true))
                .context("Failed to replay the simulation.")
        },
    }
}

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

use necsim_impls_no_std::partitioning::Partitioning;

mod args;
mod cli;
mod maps;
mod minimal_logger;
mod reporter;
mod simulation;
mod tiff;

use args::RustcoalescenceArgs;
use minimal_logger::MinimalLogger;
use reporter::RustcoalescenceReporterContext as ReporterContext;

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() -> Result<()> {
    // Parse and validate all command line arguments
    let args = RustcoalescenceArgs::from_args();

    log::set_logger(&MINIMAL_LOGGER)?;

    match args {
        RustcoalescenceArgs::Simulate(ref simulate_args) => {
            // Initialise the simulation partitioning
            let partitioning = {
                #[cfg(feature = "necsim-mpi")]
                {
                    necsim_impls_mpi::MpiPartitioning::initialise(&std::path::PathBuf::from(
                        "event_log",
                    ))
                    .with_context(|| "Failed to initialise MPI.")?
                }
                #[cfg(not(feature = "necsim-mpi"))]
                {
                    necsim_impls_no_std::partitioning::MonolithicPartitioning::default()
                }
            };

            // Only log to stderr if the partition is the root partition
            log::set_max_level(if partitioning.is_root() {
                LevelFilter::Info
            } else {
                LevelFilter::Off
            });

            info!("Parsed arguments:\n{:#?}", args);

            // Initialise the local partition and the simulation
            #[cfg(feature = "necsim-mpi")]
            {
                use necsim_impls_mpi::MpiLocalPartition;

                match partitioning.into_local_partition(ReporterContext::default()) {
                    MpiLocalPartition::Monolithic(partition) => {
                        cli::simulate::simulate_with_logger(partition, simulate_args)
                    },
                    MpiLocalPartition::Root(partition) => {
                        cli::simulate::simulate_with_logger(partition, simulate_args)
                    },
                    MpiLocalPartition::Parallel(partition) => {
                        cli::simulate::simulate_with_logger(partition, simulate_args)
                    },
                }
            }
            #[cfg(not(feature = "necsim-mpi"))]
            {
                cli::simulate::simulate_with_logger(
                    Box::new(partitioning.into_local_partition(ReporterContext::default())),
                    simulate_args,
                )
            }
        }
        .context("Failed to initialise or perform the simulation."),

        RustcoalescenceArgs::Replay(ref replay_args) => {
            // Always log to stderr (replay is run without partitioning)
            log::set_max_level(LevelFilter::Info);

            info!("Parsed arguments:\n{:#?}", args);

            cli::replay::replay_with_logger(replay_args, ReporterContext::default())
                .context("Failed to replay the simulation.")
        },
    }
}

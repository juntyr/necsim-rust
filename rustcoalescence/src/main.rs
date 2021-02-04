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
mod maps;
mod minimal_logger;
mod reporter;
mod simulation;
mod tiff;

use args::{Command, CommandLineArguments};
use minimal_logger::MinimalLogger;

static MINIMAL_LOGGER: MinimalLogger = MinimalLogger;

fn main() -> Result<()> {
    let mut partitioning = {
        #[cfg(feature = "necsim-mpi")]
        {
            necsim_impls_mpi::MpiPartitioning::initialise()
                .with_context(|| "Failed to initialise MPI.")?
        }
        #[cfg(not(feature = "necsim-mpi"))]
        {
            necsim_impls_no_std::partitioning::MonolithicPartitioning::default()
        }
    };

    log::set_logger(&MINIMAL_LOGGER)?;
    log::set_max_level(if partitioning.is_root() {
        LevelFilter::Info
    } else {
        LevelFilter::Off
    });

    main_with_logger(&mut partitioning)
}

fn main_with_logger<P: Partitioning>(partitioning: &mut P) -> Result<()> {
    // Parse and validate all command line arguments
    let args = CommandLineArguments::from_args();

    info!("Parsed arguments:\n{:#?}", args);

    anyhow::ensure!(
        *args.common_args().speciation_probability_per_generation() > 0.0_f64
            && *args.common_args().speciation_probability_per_generation() <= 1.0_f64,
        "The speciation probability per generation must be in range 0 < s <= 1."
    );

    anyhow::ensure!(
        *args.common_args().sample_percentage() >= 0.0_f64
            && *args.common_args().sample_percentage() <= 1.0_f64,
        "The sampling percentage must be in range 0 <= s <= 1."
    );

    if partitioning.is_monolithic() {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            partitioning.get_number_of_partitions().get()
        );
    }

    let (time, steps) = match args.command() {
        Command::InMemory(in_memory_args) => simulation::setup_in_memory_simulation(
            args.common_args(),
            in_memory_args,
            partitioning,
        )?,
        Command::NonSpatial(non_spatial_args) => simulation::setup_non_spatial_simulation(
            args.common_args(),
            non_spatial_args,
            partitioning,
        )?,
        Command::SpatiallyImplicit(spatially_implicit_args) => {
            simulation::setup_spatially_implicit_simulation(
                args.common_args(),
                spatially_implicit_args,
                partitioning,
            )?
        },
        Command::AlmostInfinite(almost_infinite_args) => {
            simulation::setup_almost_infinite_simulation(
                args.common_args(),
                almost_infinite_args,
                partitioning,
            )?
        },
    };

    info!("Simulation finished after {} ({} steps).", time, steps);

    Ok(())
}

#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(associated_type_bounds)]

#[macro_use]
extern crate necsim_core;

use anyhow::Result;
use structopt::StructOpt;

mod args;
mod maps;
mod reporter;
mod simulation;
mod tiff;

use args::{Command, CommandLineArguments};

fn main() -> Result<()> {
    // Parse and validate all command line arguments
    let args = CommandLineArguments::from_args();

    println!("Parsed arguments:\n{:#?}", args);

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

    let (time, steps) = match args.command() {
        Command::InMemory(in_memory_args) => {
            simulation::setup_in_memory_simulation(args.common_args(), in_memory_args)?
        },
        Command::NonSpatial(non_spatial_args) => {
            simulation::setup_non_spatial_simulation(args.common_args(), non_spatial_args)?
        },
        Command::AlmostInfinite(almost_infinite_args) => {
            simulation::setup_almost_infinite_simulation(args.common_args(), almost_infinite_args)?
        },
    };

    println!("Simulation finished after {} ({} steps).", time, steps);

    Ok(())
}

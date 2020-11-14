#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

use anyhow::Result;
use array2d::Array2D;
use structopt::StructOpt;

mod args;
mod gdal;
mod maps;
mod reporter;
mod simulation;

#[macro_use]
extern crate necsim_core;

fn main() -> Result<()> {
    // Parse and validate all command line arguments
    let args = args::CommandLineArguments::from_args();

    println!("Parsed arguments:\n{:#?}", args);

    anyhow::ensure!(
        *args.speciation_probability_per_generation() > 0.0_f64
            && *args.speciation_probability_per_generation() <= 1.0_f64,
        "The speciation probability per generation must be in range 0 < s <= 1."
    );

    anyhow::ensure!(
        *args.sample_percentage() >= 0.0_f64 && *args.sample_percentage() <= 1.0_f64,
        "The sampling percentage must be in range 0 <= s <= 1."
    );

    let dispersal: Array2D<f64> = maps::load_dispersal_map(args.dispersal_map())?;

    println!(
        "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
        args.dispersal_map(),
        dispersal.num_columns(),
        dispersal.num_rows()
    );

    let habitat: Array2D<u32> = maps::load_habitat_map(args.habitat_map(), &dispersal)?;

    println!(
        "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
        args.habitat_map(),
        habitat.num_columns(),
        habitat.num_rows()
    );

    // Run the simulation
    let (time, steps) = simulation::simulate(
        &args,
        &habitat,
        &dispersal,
        reporter::RustcoalescenceReporterContext::new(&args, &habitat),
    )?;

    println!("Simulation finished after {} ({} steps).", time, steps);

    Ok(())
}

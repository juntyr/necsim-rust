#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate necsim_core;

use anyhow::Result;
use array2d::Array2D;
use structopt::StructOpt;

mod args;
mod gdal;
mod maps;
mod reporter;
mod simulation;

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
            let dispersal: Array2D<f64> = maps::load_dispersal_map(in_memory_args.dispersal_map())?;

            println!(
                "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
                in_memory_args.dispersal_map(),
                dispersal.num_columns(),
                dispersal.num_rows()
            );

            let habitat: Array2D<u32> =
                maps::load_habitat_map(in_memory_args.habitat_map(), &dispersal)?;

            println!(
                "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
                in_memory_args.habitat_map(),
                habitat.num_columns(),
                habitat.num_rows()
            );

            let total_habitat = habitat
                .elements_row_major_iter()
                .map(|x| u64::from(*x))
                .sum::<u64>();

            #[allow(clippy::cast_possible_truncation)]
            #[allow(clippy::cast_sign_loss)]
            #[allow(clippy::cast_precision_loss)]
            let estimated_total_lineages =
                ((total_habitat as f64) * args.common_args().sample_percentage()).ceil() as u64;

            // Run the simulation
            simulation::in_memory::simulate(
                args.common_args(),
                &in_memory_args,
                &habitat,
                &dispersal,
                reporter::RustcoalescenceReporterContext::new(estimated_total_lineages),
            )?
        },
        Command::NonSpatial(non_spatial_args) => {
            let estimated_total_lineages = (f64::from(non_spatial_args.area().0)
                * f64::from(non_spatial_args.area().1)
                * f64::from(*non_spatial_args.deme())
                * args.common_args().sample_percentage())
            .ceil() as u64;

            // Run the simulation
            simulation::non_spatial::simulate(
                args.common_args(),
                &non_spatial_args,
                reporter::RustcoalescenceReporterContext::new(estimated_total_lineages),
            )?
        },
    };

    println!("Simulation finished after {} ({} steps).", time, steps);

    Ok(())
}

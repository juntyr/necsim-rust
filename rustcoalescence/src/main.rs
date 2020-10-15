#![deny(clippy::pedantic)]

use anyhow::{Context, Result};
use array2d::Array2D;
use structopt::StructOpt;

mod gdal;
mod stdrng;

use necsim_classical::ClassicalSimulation;
#[macro_use]
extern crate necsim_core;
use necsim_impls::reporter::biodiversity::BiodiversityReporter;
use necsim_impls::reporter::events::EventReporter;
use necsim_impls::reporter::execution_time::ExecutionTimeReporter;

use self::gdal::load_map_from_gdal_raster;
use stdrng::NewStdRng;

#[derive(Debug, StructOpt)]
struct CommandLineArguments {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    seed: u64,
}

fn main() -> Result<()> {
    let args = CommandLineArguments::from_args();

    println!("Parsed arguments:\n{:#?}", args);

    anyhow::ensure!(
        args.speciation_probability_per_generation > 0.0_f64
            && args.speciation_probability_per_generation <= 1.0_f64,
        "The speciation probability per generation must be in range 0 < s <= 1."
    );

    anyhow::ensure!(
        args.sample_percentage >= 0.0_f64 && args.sample_percentage <= 1.0_f64,
        "The sampling percentage must be in range 0 <= s <= 1."
    );

    let habitat: Array2D<u32> =
        load_map_from_gdal_raster(&args.habitat_map).context("Failed to load the habitat map")?;

    println!(
        "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
        args.habitat_map,
        habitat.num_columns(),
        habitat.num_rows()
    );

    let dispersal: Array2D<f64> = load_map_from_gdal_raster(&args.dispersal_map)
        .context("Failed to load the dispersal map")?;

    println!(
        "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
        args.dispersal_map,
        dispersal.num_columns(),
        dispersal.num_rows()
    );

    let mut rng = NewStdRng::from_seed(args.seed);
    let mut biodiversity_reporter = BiodiversityReporter::default();
    let mut event_reporter = EventReporter::default();
    let mut execution_time_reporter = ExecutionTimeReporter::default();

    let mut reporter_group = ReporterGroup![
        biodiversity_reporter,
        event_reporter,
        execution_time_reporter
    ];

    println!("Setting up the classical coalescence algorithm ...");

    let (time, steps) = ClassicalSimulation::simulate(
        habitat,
        &dispersal,
        args.speciation_probability_per_generation,
        args.sample_percentage,
        &mut rng,
        &mut reporter_group,
    )
    .with_context(|| {
        format!(
            concat!(
                "Failed to create a Landscape with the habitat ",
                "map {:?} and the dispersal map {:?}."
            ),
            args.dispersal_map, args.habitat_map
        )
    })?;

    let execution_time = execution_time_reporter.execution_time();

    event_reporter.report();

    println!(
        "The simulation took {}s to execute.",
        execution_time.as_secs_f32()
    );
    println!("Simulation finished after {} ({} steps).", time, steps);
    println!(
        "Simulation resulted with biodiversity of {} unique species.",
        biodiversity_reporter.biodiversity()
    );

    Ok(())
}

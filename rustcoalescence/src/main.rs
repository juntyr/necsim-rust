#![deny(clippy::pedantic)]

use anyhow::{Context, Result};
use array2d::Array2D;
use structopt::StructOpt;

mod gdal;
mod stdrng;

use necsim_classical::ClassicalSimulation;
use necsim_impls::reporter::biodiversity::BiodiversityReporter;

use self::gdal::load_map_from_gdal_raster;
use stdrng::NewStdRng;

#[derive(Debug, StructOpt)]
struct CommandLineArguments {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
    speciation_probability_per_generation: f64,
    //sample_percentage: f64, // TODO: Check [0; 1]
    seed: u64,
}

fn main() -> Result<()> {
    let args = CommandLineArguments::from_args();

    println!("Parsed arguments:\n{:#?}", args);

    anyhow::ensure!(
        args.speciation_probability_per_generation > 0.0_f64
            && args.speciation_probability_per_generation <= 1.0_f64,
        "The speciation probability per generation must be in range [0; 1)."
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

    println!("Setting up the classical coalescence algorithm ...");

    let (time, steps) = ClassicalSimulation::simulate(
        habitat,
        &args.habitat_map,
        &dispersal,
        &args.dispersal_map,
        args.speciation_probability_per_generation,
        &mut rng,
        &mut biodiversity_reporter,
    )?;

    println!("Simulation finished after {} ({} steps).", time, steps);
    println!(
        "Simulation resulted with biodiversity of {} unique species.",
        biodiversity_reporter.biodiversity()
    );

    Ok(())
}

#![deny(clippy::pedantic)]

use anyhow::{Context, Result};
use array2d::Array2D;
use gdal::raster::{Buffer, Dataset};
use structopt::StructOpt;

use necsim::{simulate, Landscape};

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf, // TODO: Check if exists
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf, // TODO: Check if exists
    speciation_probability_per_generation: f64, // TODO: Check ]0; 1[
    //sample_percentage: f64, // TODO: Check [0; 1]
    seed: u64,
}

struct GdalError(gdal::errors::Error);

impl std::fmt::Debug for GdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, fmt)
    }
}

impl std::fmt::Display for GdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

impl std::error::Error for GdalError {}

fn load_array2d_from_gdal_raster<T: Copy + gdal::raster::types::GdalType>(
    path: &std::path::PathBuf,
) -> Result<Array2D<T>> {
    let dataset = Dataset::open(path)
        .map_err(GdalError)
        .with_context(|| format!("The map file {:?} could not be opened.", path))?;

    let data: Buffer<T> = dataset
        .read_full_raster_as(1)
        .map_err(GdalError)
        .with_context(|| format!("The map file {:?} could not be read.", path))?;

    Ok(Array2D::from_row_major(
        &data.data,
        data.size.1,
        data.size.0,
    ))
}

fn main() -> Result<()> {
    let args = Cli::from_args();

    println!("Simulation args: {:#?}", args);

    let habitat: Array2D<u32> = load_array2d_from_gdal_raster(&args.habitat_map)
        .context("Failed to load the habitat map")?;

    println!(
        "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
        args.habitat_map,
        habitat.num_columns(),
        habitat.num_rows()
    );

    let dispersal: Array2D<f64> = load_array2d_from_gdal_raster(&args.dispersal_map)
        .context("Failed to load the dispersal map")?;

    println!(
        "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
        args.dispersal_map,
        dispersal.num_columns(),
        dispersal.num_rows()
    );

    let landscape = Landscape::new(habitat, &dispersal).with_context(|| {
        format!(
            concat!(
                "Failed to create a Landscape with the habitat ",
                "map {:?} and the dispersal map {:?}."
            ),
            args.dispersal_map, args.habitat_map
        )
    })?;

    println!("Starting the simulation ...");

    let biodiversity = simulate(
        args.speciation_probability_per_generation,
        landscape,
        args.seed,
    );

    println!("Resulting biodiversity: {}", biodiversity);

    Ok(())
}

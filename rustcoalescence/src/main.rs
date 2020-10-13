#![deny(clippy::pedantic)]

use anyhow::{Context, Result};
use array2d::Array2D;
use gdal::raster::{Buffer, Dataset};
use rand::SeedableRng;
use structopt::StructOpt;

use necsim::{
    landscape::impls::LandscapeInMemoryWithPrecalculatedDispersal,
    simulation::settings::SimulationSettings, simulation::Simulation,
};

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

struct NewGdalError(gdal::errors::Error);

impl std::fmt::Debug for NewGdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Debug::fmt(&self.0, fmt)
    }
}

impl std::fmt::Display for NewGdalError {
    fn fmt(&self, fmt: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(&self.0, fmt)
    }
}

impl std::error::Error for NewGdalError {}

struct NewStdRng(rand::rngs::StdRng);

impl necsim::rng::Core for NewStdRng {
    fn sample_uniform(&mut self) -> f64 {
        use rand::Rng;

        self.0.gen_range(0.0_f64, 1.0_f64)
    }
}

fn load_array2d_from_gdal_raster<T: Copy + gdal::raster::types::GdalType>(
    path: &std::path::PathBuf,
) -> Result<Array2D<T>> {
    let dataset = Dataset::open(path)
        .map_err(NewGdalError)
        .with_context(|| format!("The map file {:?} could not be opened.", path))?;

    let data: Buffer<T> = dataset
        .read_full_raster_as(1)
        .map_err(NewGdalError)
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

    let landscape = LandscapeInMemoryWithPrecalculatedDispersal::new(habitat, &dispersal)
        .with_context(|| {
            format!(
                concat!(
                    "Failed to create a Landscape with the habitat ",
                    "map {:?} and the dispersal map {:?}."
                ),
                args.dispersal_map, args.habitat_map
            )
        })?;

    let settings = SimulationSettings::new(args.speciation_probability_per_generation, landscape);
    let rng = rand::rngs::StdRng::seed_from_u64(args.seed);

    let biodiversity = Simulation::simulate(&settings, &mut NewStdRng(rng));

    println!("Resulting biodiversity: {}", biodiversity);

    Ok(())
}

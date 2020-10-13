use array2d::Array2D;
use gdal::raster::{Buffer, Dataset};
use structopt::StructOpt;

use necsim::{simulate, InconsistentDispersalMapSize, Landscape};

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

fn load_array2d_from_gdal_raster<T: Copy + gdal::raster::types::GdalType>(
    path: &std::path::PathBuf,
) -> Array2D<T> {
    let habitat_dataset = match Dataset::open(path) {
        Ok(dataset) => dataset,
        Err(_) => {
            println!("Error: The map file '{:?}' could not be opened.", path);
            std::process::abort()
        }
    };

    let habitat_data: Buffer<T> = match habitat_dataset.read_full_raster_as(1) {
        Ok(data) => data,
        Err(_) => {
            println!("Error: The map file '{:?}' could not be read.", path);
            std::process::abort()
        }
    };

    Array2D::from_row_major(&habitat_data.data, habitat_data.size.0, habitat_data.size.1)
}

fn main() {
    let args = Cli::from_args();

    println!("Simulation args: {:?}", args);

    let habitat: Array2D<u32> = load_array2d_from_gdal_raster(&args.habitat_map);
    let dispersal: Array2D<f64> = load_array2d_from_gdal_raster(&args.dispersal_map);

    let landscape = match Landscape::new(habitat, &dispersal) {
        Ok(landscape) => landscape,
        Err(InconsistentDispersalMapSize) => {
            println!(
                concat!(
                    "Error: The size of the dispersal map '{:?}' is inconsistent ",
                    "with the size of the habitat map '{:?}'."
                ),
                args.dispersal_map, args.habitat_map
            );
            std::process::abort()
        }
    };

    let biodiversity = simulate(
        args.speciation_probability_per_generation,
        landscape,
        args.seed,
    );

    println!("Resulting biodiversity: {}", biodiversity);
}

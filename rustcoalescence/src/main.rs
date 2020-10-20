#![deny(clippy::pedantic)]

#[macro_use]
extern crate contracts;

use anyhow::{Context, Result};
use array2d::Array2D;
use cast::{Error, From as _From};
use structopt::StructOpt;

use structopt::clap::arg_enum;

mod gdal;
mod stdrng;

use necsim_classical::ClassicalSimulation;
use necsim_gillespie::GillespieSimulation;
#[macro_use]
extern crate necsim_core;
use necsim_impls::reporter::biodiversity::BiodiversityReporter;
use necsim_impls::reporter::events::EventReporter;
use necsim_impls::reporter::execution_time::ExecutionTimeReporter;
use necsim_impls::reporter::progress::ProgressReporter;

use self::gdal::load_map_f64_from_gdal_raster;
use stdrng::NewStdRng;

arg_enum! {
    #[derive(Debug)]
    enum Algorithm {
        Classical,
        Gillespie
    }
}

#[derive(Debug, StructOpt)]
struct CommandLineArguments {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
    speciation_probability_per_generation: f64,
    sample_percentage: f64,
    seed: u64,
    #[structopt(possible_values = &Algorithm::variants(), case_insensitive = true)]
    algorithm: Algorithm,
}

#[allow(clippy::too_many_lines)] // TODO: Refactor
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

    let habitat_f64 = load_map_f64_from_gdal_raster(&args.habitat_map)
        .context("Failed to load the habitat map")?;

    let mut habitat: Array2D<u32> =
        Array2D::filled_with(0, habitat_f64.num_rows(), habitat_f64.num_columns());

    for y in 0..habitat_f64.num_rows() {
        for x in 0..habitat_f64.num_columns() {
            let h_f64 = habitat_f64[(y, x)];

            habitat[(y, x)] = if h_f64 < 0.0_f64 {
                Err(Error::Underflow)
            } else {
                u32::cast(h_f64.ceil())
            }
            .context("Failed to interpret the habitat map as u32")?;
        }
    }
    println!(
        "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
        args.habitat_map,
        habitat.num_columns(),
        habitat.num_rows()
    );

    let dispersal: Array2D<f64> = load_map_f64_from_gdal_raster(&args.dispersal_map)
        .context("Failed to load the dispersal map")?;

    println!(
        "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
        args.dispersal_map,
        dispersal.num_columns(),
        dispersal.num_rows()
    );

    let total_habitat = habitat
        .elements_row_major_iter()
        .map(|x| u64::from(*x))
        .sum::<u64>();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    let estimated_total_lineages = ((total_habitat as f64) * args.sample_percentage).ceil() as u64;

    let mut rng = NewStdRng::from_seed(args.seed);
    let mut biodiversity_reporter = BiodiversityReporter::default();
    let mut event_reporter = EventReporter::default();
    let mut execution_time_reporter = ExecutionTimeReporter::default();
    let mut progress_reporter = ProgressReporter::new(estimated_total_lineages);

    let mut reporter_group = ReporterGroup![
        biodiversity_reporter,
        event_reporter,
        execution_time_reporter,
        progress_reporter
    ];

    println!(
        "Setting up the {:?} coalescence algorithm ...",
        args.algorithm
    );

    let (time, steps) = match args.algorithm {
        Algorithm::Classical => ClassicalSimulation::simulate(
            habitat,
            &dispersal,
            args.speciation_probability_per_generation,
            args.sample_percentage,
            &mut rng,
            &mut reporter_group,
        ),
        Algorithm::Gillespie => GillespieSimulation::simulate(
            habitat,
            &dispersal,
            args.speciation_probability_per_generation,
            args.sample_percentage,
            &mut rng,
            &mut reporter_group,
        ),
    }
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

    progress_reporter.finish();

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

#![deny(clippy::pedantic)]

use anyhow::Result;
use array2d::Array2D;
use structopt::StructOpt;

mod args;
mod gdal;
mod maps;
mod simulation;

#[macro_use]
extern crate necsim_core;

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::wyhash::WyHash as Rng;
use necsim_impls_std::reporter::biodiversity::BiodiversityReporter;
/*use necsim_impls_std::reporter::events::EventReporter;
use necsim_impls_std::reporter::execution_time::ExecutionTimeReporter;*/
use necsim_impls_std::reporter::progress::ProgressReporter;

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

    // Initialise the reporters
    let total_habitat = habitat
        .elements_row_major_iter()
        .map(|x| u64::from(*x))
        .sum::<u64>();

    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    #[allow(clippy::cast_precision_loss)]
    let estimated_total_lineages =
        ((total_habitat as f64) * args.sample_percentage()).ceil() as u64;

    let mut biodiversity_reporter = BiodiversityReporter::default();
    /*let mut event_reporter = EventReporter::default();
    let mut execution_time_reporter = ExecutionTimeReporter::default();*/
    let mut progress_reporter = ProgressReporter::new(estimated_total_lineages);

    let mut reporter_group = ReporterGroup![
        biodiversity_reporter,
        //event_reporter,
        //execution_time_reporter,
        progress_reporter
    ];

    // Run the simulation
    let (time, steps) = simulation::simulate(
        &args,
        &habitat,
        &dispersal,
        Rng::seed_from_u64(*args.seed()),
        &mut reporter_group,
    )?;

    // Output the simulation result and report summaries
    //let execution_time = execution_time_reporter.execution_time();

    progress_reporter.finish();
    /*event_reporter.report();

    println!(
        "The simulation took {}s to execute.",
        execution_time.as_secs_f32()
    );*/
    println!("Simulation finished after {} ({} steps).", time, steps);
    println!(
        "Simulation resulted with biodiversity of {} unique species.",
        biodiversity_reporter.biodiversity()
    );

    Ok(())
}

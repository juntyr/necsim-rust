use anyhow::Result;

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

use crate::{
    args::{Command, SimulateArgs},
    simulation,
};

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger<R: ReporterContext, P: LocalPartition<R>>(
    mut local_partition: Box<P>,
    simulate_args: &SimulateArgs,
) -> Result<()> {
    // Parse and validate all command line arguments for the simulate subcommand
    anyhow::ensure!(
        *simulate_args
            .common_args()
            .speciation_probability_per_generation()
            > 0.0_f64
            && *simulate_args
                .common_args()
                .speciation_probability_per_generation()
                <= 1.0_f64,
        "The speciation probability per generation must be in range 0 < s <= 1."
    );

    anyhow::ensure!(
        *simulate_args.common_args().sample_percentage() >= 0.0_f64
            && *simulate_args.common_args().sample_percentage() <= 1.0_f64,
        "The sampling percentage must be in range 0 <= s <= 1."
    );

    if local_partition.get_number_of_partitions().get() <= 1 {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            local_partition.get_number_of_partitions().get()
        );
    }

    let (time, steps) = match simulate_args.command() {
        Command::InMemory(in_memory_args) => simulation::setup_in_memory_simulation(
            simulate_args.common_args(),
            in_memory_args,
            local_partition.as_mut(),
        )?,
        Command::NonSpatial(non_spatial_args) => simulation::setup_non_spatial_simulation(
            simulate_args.common_args(),
            non_spatial_args,
            local_partition.as_mut(),
        )?,
        Command::SpatiallyImplicit(spatially_implicit_args) => {
            simulation::setup_spatially_implicit_simulation(
                simulate_args.common_args(),
                spatially_implicit_args,
                local_partition.as_mut(),
            )?
        },
        Command::AlmostInfinite(almost_infinite_args) => {
            simulation::setup_almost_infinite_simulation(
                simulate_args.common_args(),
                almost_infinite_args,
                local_partition.as_mut(),
            )?
        },
    };

    std::mem::drop(local_partition);

    info!("Simulation finished after {} ({} steps).", time, steps);

    Ok(())
}

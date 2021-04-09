use anyhow::Result;

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

use crate::{
    args::{CommonArgs, Scenario},
    simulation,
};

#[cfg(not(feature = "necsim-mpi"))]
pub mod monolithic;
#[cfg(feature = "necsim-mpi")]
pub mod mpi;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger<R: ReporterContext, P: LocalPartition<R>>(
    mut local_partition: Box<P>,
    common_args: CommonArgs,
    scenario: Scenario,
) -> Result<()> {
    if local_partition.get_number_of_partitions().get() <= 1 {
        info!("The simulation will be run in monolithic mode.");
    } else {
        info!(
            "The simulation will be distributed across {} partitions.",
            local_partition.get_number_of_partitions().get()
        );
    }

    let (time, steps) = match scenario {
        Scenario::InMemory(in_memory_args) => {
            simulation::in_memory::simulate(common_args, in_memory_args, local_partition.as_mut())?
        },
        Scenario::NonSpatial(non_spatial_args) => simulation::non_spatial::simulate(
            common_args,
            non_spatial_args,
            local_partition.as_mut(),
        )?,
        Scenario::SpatiallyImplicit(spatially_implicit_args) => {
            simulation::spatially_implicit::simulate(
                common_args,
                spatially_implicit_args,
                local_partition.as_mut(),
            )?
        },
        Scenario::AlmostInfinite(almost_infinite_args) => simulation::almost_infinite::simulate(
            common_args,
            almost_infinite_args,
            local_partition.as_mut(),
        )?,
    };

    if log::log_enabled!(log::Level::Info) {
        eprintln!("\n\n{:=^80}\n", " Reporter Summary ")
    };
    local_partition.finalise_reporting();
    if log::log_enabled!(log::Level::Info) {
        eprintln!("\n{:=^80}\n", " Reporter Summary ")
    };

    info!("Simulation finished after {} ({} steps).\n", time, steps);

    Ok(())
}

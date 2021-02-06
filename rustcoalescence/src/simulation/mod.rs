use anyhow::Result;
use array2d::Array2D;

use necsim_impls_no_std::{partitioning::LocalPartition, reporter::ReporterContext};

use crate::{
    args::{AlmostInfiniteArgs, CommonArgs, InMemoryArgs, NonSpatialArgs, SpatiallyImplicitArgs},
    maps,
};

mod almost_infinite;
mod in_memory;
mod non_spatial;
mod spatially_implicit;

#[allow(clippy::module_name_repetitions)]
pub fn setup_in_memory_simulation<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    in_memory_args: &InMemoryArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    let dispersal: Array2D<f64> = maps::load_dispersal_map(
        in_memory_args.dispersal_map(),
        *in_memory_args.strict_load(),
    )?;

    info!(
        "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
        in_memory_args.dispersal_map(),
        dispersal.num_columns(),
        dispersal.num_rows()
    );

    let habitat: Array2D<u32> = maps::load_habitat_map(
        in_memory_args.habitat_map(),
        &dispersal,
        *in_memory_args.strict_load(),
    )?;

    info!(
        "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
        in_memory_args.habitat_map(),
        habitat.num_columns(),
        habitat.num_rows()
    );

    // Run the simulation
    in_memory::simulate(
        common_args,
        &in_memory_args,
        &habitat,
        &dispersal,
        local_partition,
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn setup_non_spatial_simulation<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    non_spatial_args: &NonSpatialArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    if *non_spatial_args.spatial() {
        return setup_non_spatial_in_memory_simulation(
            common_args,
            non_spatial_args,
            local_partition,
        );
    }

    // Run the simulation
    non_spatial::simulate(common_args, &non_spatial_args, local_partition)
}

fn setup_non_spatial_in_memory_simulation<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    non_spatial_args: &NonSpatialArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    let habitat = Array2D::filled_with(
        *non_spatial_args.deme(),
        non_spatial_args.area().1 as usize,
        non_spatial_args.area().0 as usize,
    );

    let total_area = (non_spatial_args.area().0 as usize) * (non_spatial_args.area().1 as usize);

    let dispersal = Array2D::filled_with(1.0_f64, total_area, total_area);

    // Run the simulation
    in_memory::simulate(
        common_args,
        &non_spatial_args.as_in_memory(),
        &habitat,
        &dispersal,
        local_partition,
    )
}

#[allow(clippy::module_name_repetitions)]
pub fn setup_spatially_implicit_simulation<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    spatially_implicit_args: &SpatiallyImplicitArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    // Run the simulation
    spatially_implicit::simulate(common_args, &spatially_implicit_args, local_partition)
}

#[allow(clippy::module_name_repetitions)]
pub fn setup_almost_infinite_simulation<R: ReporterContext, P: LocalPartition<R>>(
    common_args: &CommonArgs,
    almost_infinite_args: &AlmostInfiniteArgs,
    local_partition: &mut P,
) -> Result<(f64, u64)> {
    anyhow::ensure!(
        *almost_infinite_args.sigma() >= 0.0_f64,
        "The dispersal standard deviation must be non-negative."
    );

    // Run the simulation
    almost_infinite::simulate(common_args, &almost_infinite_args, local_partition)
}

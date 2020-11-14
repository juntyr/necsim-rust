use std::ffi::CString;

use anyhow::Result;

use rustacuda::{
    function::{BlockSize, Function, GridSize},
    module::Symbol,
    prelude::*,
};

use rust_cuda::host::LendToCuda;

use necsim_core::{cogs::LineageStore, reporter::Reporter};

use necsim_impls_cuda::event_buffer::host::EventBufferHost;

use crate::type_checked_launch;

pub fn simulate<P: Reporter<config::Habitat, config::LineageReference>>(
    mut simulation: config::Simulation,
    max_steps: u64,
    mut event_buffer: EventBufferHost<
        config::Habitat,
        config::LineageReference,
        P,
        { config::REPORT_SPECIATION },
        { config::REPORT_DISPERSAL },
    >,
    stream: &Stream,
    module: &Module,
    simulate: &Function,
    task: (u32, GridSize, BlockSize),
) -> Result<(f64, u64)> {
    // Load and initialise the grid_id symbol from the module
    let mut grid_id_symbol: Symbol<u32> = module.get_global(&CString::new("grid_id").unwrap())?;
    grid_id_symbol.copy_from(&0_u32)?;

    // Load and initialise the global_lineages_remaining symbol
    let mut global_lineages_remaining =
        simulation.lineage_store().get_number_total_lineages() as u64;
    let mut global_lineages_remaining_symbol: Symbol<u64> =
        module.get_global(&CString::new("global_lineages_remaining").unwrap())?;
    global_lineages_remaining_symbol.copy_from(&global_lineages_remaining)?;

    // Load and initialise the global_time_max and global_steps_sum symbols
    let mut global_time_max_symbol: Symbol<f64> =
        module.get_global(&CString::new("global_time_max").unwrap())?;
    global_time_max_symbol.copy_from(&0.0_f64)?;
    let mut global_steps_sum_symbol: Symbol<u64> =
        module.get_global(&CString::new("global_steps_sum").unwrap())?;
    global_steps_sum_symbol.copy_from(&0_u64)?;

    let (grid_amount, grid_size, block_size) = task;

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    if let Err(err) = simulation.lend_to_cuda_mut(|simulation_mut_ptr| {
        let mut time_slice = 0;

        while global_lineages_remaining > 0 {
            println!(
                "Starting time slice {} with {} remaining individuals ({} grids) ...",
                time_slice + 1,
                global_lineages_remaining,
                grid_amount
            );

            for grid_id in 0..grid_amount {
                grid_id_symbol.copy_from(&grid_id)?;

                let grid_size = grid_size.clone();
                let block_size = block_size.clone();

                // Launching kernels is unsafe since Rust cannot enforce safety across
                // the foreign function CUDA-C language barrier
                unsafe {
                    type_checked_launch!(
                        simulate<<<grid_size, block_size, 0, stream>>>(
                            simulation_mut_ptr: rustacuda_core::DevicePointer<
                                <config::Simulation as rust_cuda::common::RustToCuda>
                                    ::CudaRepresentation
                            > = simulation_mut_ptr,
                        event_buffer_ptr: rustacuda_core::DevicePointer<
                            config::EventBufferCudaRepresentation
                        > = event_buffer.get_mut_cuda_ptr(),
                        max_steps: u64 = max_steps
                    ))?;
                }

                stream.synchronize()?;

                global_lineages_remaining_symbol.copy_to(&mut global_lineages_remaining)?;

                event_buffer.fetch_and_report_events()?
            }

            time_slice += 1;
        }

        Ok(())
    }) {
        eprintln!("\nRunning kernel failed with {:#?}!\n", err);
    }

    let mut global_time_max = 0.0_f64;
    let mut global_steps_sum = 0_u64;

    global_time_max_symbol.copy_to(&mut global_time_max)?;
    global_steps_sum_symbol.copy_to(&mut global_steps_sum)?;

    Ok((global_time_max, global_steps_sum))
}

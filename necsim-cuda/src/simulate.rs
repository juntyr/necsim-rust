use std::ffi::CString;

use anyhow::Result;

use rustacuda::{
    function::{BlockSize, GridSize},
    memory::CopyDestination,
    module::Symbol,
    stream::Stream,
};

use rust_cuda::{common::RustToCuda, host::LendToCuda};
use rustacuda_core::DeviceCopy;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
        HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
    },
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_cuda::event_buffer::host::EventBufferHost;

use crate::kernel::SimulationKernel;

pub fn simulate<
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter<H, R>,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: EventSampler<H, G, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    stream: &Stream,
    kernel: &SimulationKernel<H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
    task: (u32, GridSize, BlockSize),
    mut simulation: Simulation<H, G, D, R, S, C, E, A>,
    mut event_buffer: EventBufferHost<H, R, P, REPORT_SPECIATION, REPORT_DISPERSAL>,
    max_steps: u64,
) -> Result<(f64, u64)> {
    // Load and initialise the grid_id symbol from the kernel
    let mut grid_id_symbol: Symbol<u32> = kernel.get_global(&CString::new("grid_id").unwrap())?;
    grid_id_symbol.copy_from(&0_u32)?;

    // Load and initialise the global_lineages_remaining symbol
    let mut global_lineages_remaining =
        simulation.lineage_store().get_number_total_lineages() as u64;
    let mut global_lineages_remaining_symbol: Symbol<u64> =
        kernel.get_global(&CString::new("global_lineages_remaining").unwrap())?;
    global_lineages_remaining_symbol.copy_from(&global_lineages_remaining)?;

    // Load and initialise the global_time_max and global_steps_sum symbols
    let mut global_time_max_symbol: Symbol<f64> =
        kernel.get_global(&CString::new("global_time_max").unwrap())?;
    global_time_max_symbol.copy_from(&0.0_f64)?;
    let mut global_steps_sum_symbol: Symbol<u64> =
        kernel.get_global(&CString::new("global_steps_sum").unwrap())?;
    global_steps_sum_symbol.copy_from(&0_u64)?;

    let (grid_amount, grid_size, block_size) = task;

    let kernel = kernel
        .with_dimensions(grid_size, block_size, 0_u32)
        .with_stream(stream);

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

                // Launching kernels is unsafe since Rust cannot enforce safety across
                // the foreign function CUDA-C language barrier
                unsafe {
                    kernel.launch(
                        simulation_mut_ptr,
                        event_buffer.get_mut_cuda_ptr(),
                        max_steps,
                    )?;
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

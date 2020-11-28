use std::{
    collections::{HashSet, VecDeque},
    ffi::CString,
};

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
        CoalescenceSampler, DispersalSampler, HabitatToU64Injection, IncoherentLineageStore,
        LineageReference, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationSample,
    },
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_cuda::{
    event_buffer::host::EventBufferHost, task_list::host::TaskListHost,
    value_buffer::host::ValueBufferHost,
};

use crate::kernel::SimulationKernel;

#[allow(clippy::too_many_arguments)]
pub fn simulate<
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter<H, R>,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, D, R, S, C> + RustToCuda,
    A: SingularActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    stream: &Stream,
    kernel: &SimulationKernel<H, G, D, R, S, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
    task: (u32, GridSize, BlockSize),
    mut simulation: Simulation<H, G, D, R, S, C, E, A>,
    mut task_list: TaskListHost<H, R>,
    mut event_buffer: EventBufferHost<H, R, P, REPORT_SPECIATION, REPORT_DISPERSAL>,
    mut min_spec_sample_buffer: ValueBufferHost<SpeciationSample>,
    max_steps: u64,
) -> Result<(f64, u64)> {
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

    let mut individual_tasks: VecDeque<R> = simulation
        .lineage_store()
        .iter_local_lineage_references()
        .collect();

    let mut min_spec_samples: HashSet<SpeciationSample> = HashSet::new();
    let mut duplicate_individuals = bitbox![0; min_spec_sample_buffer.len()];

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    if let Err(err) = simulation.lend_to_cuda_mut(|simulation_mut_ptr| {
        while !individual_tasks.is_empty() {
            for _grid_id in 0..(grid_amount as usize) {
                task_list.with_upload_and_fetch_tasks(
                    &mut (
                        &mut individual_tasks,
                        &mut min_spec_samples,
                        &mut duplicate_individuals,
                    ),
                    |(individual_tasks, ..), new_tasks| {
                        // Upload the new tasks
                        for task in new_tasks {
                            *task = individual_tasks.pop_front();
                        }
                    },
                    |(_, min_spec_samples, duplicate_individuals), task_list_mut_ptr| {
                        min_spec_sample_buffer.with_upload_and_fetch_values(
                            &mut (min_spec_samples, duplicate_individuals),
                            |(_, duplicate_individuals), new_min_spec_samples| {
                                new_min_spec_samples.fill(None);
                                duplicate_individuals.set_all(false);
                            },
                            |min_spec_sample_buffer_ptr| {
                                // Launching kernels is unsafe since Rust cannot
                                // enforce safety across the foreign function
                                // CUDA-C language barrier
                                unsafe {
                                    kernel.launch(
                                        simulation_mut_ptr,
                                        task_list_mut_ptr,
                                        event_buffer.get_mut_cuda_ptr(),
                                        min_spec_sample_buffer_ptr,
                                        max_steps,
                                    )?;
                                }

                                stream.synchronize()
                            },
                            |(min_spec_samples, duplicate_individuals), slice_min_spec_samples| {
                                for (i, spec_sample) in
                                    slice_min_spec_samples.iter_mut().enumerate()
                                {
                                    if let Some(spec_sample) = spec_sample.take() {
                                        duplicate_individuals
                                            .set(i, !min_spec_samples.insert(spec_sample));
                                    }
                                }
                            },
                        )
                    },
                    |(individual_tasks, _, duplicate_individuals), completed_tasks| {
                        // Fetch the completion of the tasks
                        for (i, task) in completed_tasks.iter_mut().enumerate() {
                            if let Some(task) = task.take() {
                                if !duplicate_individuals[i] {
                                    individual_tasks.push_back(task);
                                }
                            }
                        }
                    },
                )?;

                event_buffer.fetch_and_report_events()?
            }
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

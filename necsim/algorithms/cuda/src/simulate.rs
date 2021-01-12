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

use rust_cuda::{
    common::RustToCuda, host::LendToCuda, utils::exchange::wrapper::ExchangeWithCudaWrapper,
};
use rustacuda_core::DeviceCopy;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, IncoherentLineageStore,
        LineageReference, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    reporter::Reporter,
    simulation::Simulation,
};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use crate::kernel::SimulationKernel;

#[allow(clippy::too_many_arguments)]
pub fn simulate<
    H: Habitat + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    P: Reporter,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
    A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    stream: &Stream,
    kernel: &SimulationKernel<H, G, N, D, R, S, X, C, E, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
    task: (u32, GridSize, BlockSize),
    mut simulation: Simulation<H, G, N, D, R, S, X, C, E, A>,
    task_list: ValueBuffer<R>,
    event_buffer: EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>,
    min_spec_sample_buffer: ValueBuffer<SpeciationSample>,
    reporter: &mut P,
    max_steps: u64,
) -> Result<(f64, u64)> {
    // TODO: Remove once debugging data structure layout is no longer necessary
    use type_layout::TypeLayout;
    println!(
        "{}",
        Simulation::<H, G, N, D, R, S, X, C, E, A>::type_layout()
    );
    println!("{}", necsim_core::event::Event::type_layout());

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

    let mut task_list = ExchangeWithCudaWrapper::new(task_list)?;
    let mut event_buffer = ExchangeWithCudaWrapper::new(event_buffer)?;
    let mut min_spec_sample_buffer = ExchangeWithCudaWrapper::new(min_spec_sample_buffer)?;

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    if let Err(err) = simulation.lend_to_cuda_mut(|simulation_cuda_repr| {
        while !individual_tasks.is_empty() {
            for _grid_id in 0..(grid_amount as usize) {
                // Upload the new tasks from the front of the task queue
                for task in task_list.iter_mut() {
                    *task = individual_tasks.pop_front();
                }

                // Reset the individual duplication check bitmask
                duplicate_individuals.set_all(false);

                // Move the task list, event buffer and min speciation sample buffer to CUDA
                let mut event_buffer_cuda = event_buffer.move_to_cuda()?;
                let mut min_spec_sample_buffer_cuda = min_spec_sample_buffer.move_to_cuda()?;
                let mut task_list_cuda = task_list.move_to_cuda()?;

                // Launching kernels is unsafe since Rust cannot
                // enforce safety across the foreign function
                // CUDA-C language barrier
                unsafe {
                    kernel.launch(
                        simulation_cuda_repr,
                        task_list_cuda.as_mut(),
                        event_buffer_cuda.as_mut(),
                        min_spec_sample_buffer_cuda.as_mut(),
                        max_steps,
                    )?;
                }

                stream.synchronize()?;

                min_spec_sample_buffer = min_spec_sample_buffer_cuda.move_to_host()?;
                task_list = task_list_cuda.move_to_host()?;
                event_buffer = event_buffer_cuda.move_to_host()?;

                // Fetch the completion of the tasks
                for (i, spec_sample) in min_spec_sample_buffer.iter_mut().enumerate() {
                    if let Some(spec_sample) = spec_sample.take() {
                        duplicate_individuals.set(i, !min_spec_samples.insert(spec_sample));
                    }
                }

                // Fetch the completion of the tasks
                for (i, task) in task_list.iter_mut().enumerate() {
                    if let Some(task) = task.take() {
                        if !duplicate_individuals[i] {
                            individual_tasks.push_back(task);
                        }
                    }
                }

                event_buffer.report_events(reporter);
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

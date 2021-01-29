use std::{collections::VecDeque, ffi::CString};

use anyhow::Result;

use lru::LruCache;

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
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    lineage::Lineage,
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
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E, I> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    stream: &Stream,
    kernel: SimulationKernel<H, G, N, D, R, S, X, C, E, I, A, REPORT_SPECIATION, REPORT_DISPERSAL>,
    config: (GridSize, BlockSize),
    mut simulation: Simulation<H, G, N, D, R, S, X, C, E, I, A>,
    mut individual_tasks: VecDeque<Lineage>,
    task_list: ValueBuffer<Lineage>,
    event_buffer: EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>,
    min_spec_sample_buffer: ValueBuffer<SpeciationSample>,
    reporter: &mut P,
    max_steps: u64,
) -> Result<(f64, u64)> {
    // TODO: Remove once debugging data structure layout is no longer necessary
    use type_layout::TypeLayout;
    println!(
        "{}",
        Simulation::<H, G, N, D, R, S, X, C, E, I, A>::type_layout()
    );
    println!("{}", necsim_core::event::Event::type_layout());

    // Load and initialise the global_time_max and global_steps_sum symbols
    let mut global_time_max_symbol: Symbol<f64> =
        kernel.get_global(&CString::new("global_time_max").unwrap())?;
    global_time_max_symbol.copy_from(&0.0_f64)?;
    let mut global_steps_sum_symbol: Symbol<u64> =
        kernel.get_global(&CString::new("global_steps_sum").unwrap())?;
    global_steps_sum_symbol.copy_from(&0_u64)?;

    let (grid_size, block_size) = config;

    let mut kernel = kernel
        .with_dimensions(grid_size, block_size, 0_u32)
        .with_stream(stream);

    let mut min_spec_samples: LruCache<SpeciationSample, ()> =
        LruCache::new(individual_tasks.len() * 5);
    let mut duplicate_individuals = bitbox![0; min_spec_sample_buffer.len()];

    let mut task_list = ExchangeWithCudaWrapper::new(task_list)?;
    let mut event_buffer = ExchangeWithCudaWrapper::new(event_buffer)?;
    let mut min_spec_sample_buffer = ExchangeWithCudaWrapper::new(min_spec_sample_buffer)?;

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    if let Err(err) = simulation.lend_to_cuda_mut(|simulation_cuda_repr| {
        while !individual_tasks.is_empty() {
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
                kernel.launch_and_synchronise(
                    simulation_cuda_repr,
                    task_list_cuda.as_mut(),
                    event_buffer_cuda.as_mut(),
                    min_spec_sample_buffer_cuda.as_mut(),
                    max_steps,
                )?;
            }

            min_spec_sample_buffer = min_spec_sample_buffer_cuda.move_to_host()?;
            task_list = task_list_cuda.move_to_host()?;
            event_buffer = event_buffer_cuda.move_to_host()?;

            // Fetch the completion of the tasks
            for (i, spec_sample) in min_spec_sample_buffer.iter_mut().enumerate() {
                if let Some(spec_sample) = spec_sample.take() {
                    duplicate_individuals.set(i, min_spec_samples.put(spec_sample, ()).is_some());
                }
            }

            // Fetch the completion of the tasks
            for (i, task) in task_list.iter_mut().enumerate() {
                if let Some(task) = task.take() {
                    if task.is_active() && !duplicate_individuals[i] {
                        individual_tasks.push_back(task);
                    }
                }
            }

            event_buffer.report_events(reporter);
            reporter.report_progress(individual_tasks.len() as u64);
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

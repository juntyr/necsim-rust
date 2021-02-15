use std::collections::VecDeque;

use anyhow::Result;

use linked_hash_map::LinkedHashMap;

use rustacuda::{
    function::{BlockSize, GridSize},
    memory::{CopyDestination, DeviceBox},
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
    'k,
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
    kernel: &'k mut SimulationKernel<
        'k,
        H,
        G,
        N,
        D,
        R,
        S,
        X,
        C,
        E,
        I,
        A,
        REPORT_SPECIATION,
        REPORT_DISPERSAL,
    >,
    config: (GridSize, BlockSize),
    mut simulation: Simulation<H, G, N, D, R, S, X, C, E, I, A>,
    mut individual_tasks: VecDeque<Lineage>,
    task_list: ValueBuffer<Lineage>,
    event_buffer: EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL>,
    min_spec_sample_buffer: ValueBuffer<SpeciationSample>,
    reporter: &mut P,
    max_steps: u64,
) -> Result<(f64, u64)> {
    // Allocate and initialise the total_time_max and total_steps_sum atomics
    let mut total_time_max = DeviceBox::new(&0.0_f64.to_bits())?;
    let mut total_steps_sum = DeviceBox::new(&0_u64)?;

    let (grid_size, block_size) = config;

    let mut kernel = kernel
        .with_dimensions(grid_size, block_size, 0_u32)
        .with_stream(stream);

    let min_spec_memory_size = individual_tasks.len() * 2;
    let mut min_spec_samples: LinkedHashMap<SpeciationSample, ()> =
        LinkedHashMap::with_capacity(min_spec_memory_size);

    let mut duplicate_individuals = bitbox![0; min_spec_sample_buffer.len()];

    let mut task_list = ExchangeWithCudaWrapper::new(task_list)?;
    let mut event_buffer = ExchangeWithCudaWrapper::new(event_buffer)?;
    let mut min_spec_sample_buffer = ExchangeWithCudaWrapper::new(min_spec_sample_buffer)?;

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    if let Err(err) = simulation.lend_to_cuda_mut(|mut simulation_cuda_repr| {
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
                    &mut simulation_cuda_repr,
                    &mut task_list_cuda.as_mut(),
                    &mut event_buffer_cuda.as_mut(),
                    &mut min_spec_sample_buffer_cuda.as_mut(),
                    &mut total_time_max,
                    &mut total_steps_sum,
                    max_steps,
                )?;
            }

            min_spec_sample_buffer = min_spec_sample_buffer_cuda.move_to_host()?;
            task_list = task_list_cuda.move_to_host()?;
            event_buffer = event_buffer_cuda.move_to_host()?;

            // Fetch the completion of the tasks
            for (i, spec_sample) in min_spec_sample_buffer.iter_mut().enumerate() {
                if let Some(spec_sample) = spec_sample.take() {
                    if min_spec_samples.insert(spec_sample, ()).is_some() {
                        duplicate_individuals.set(i, true);
                    } else if min_spec_samples.len() >= min_spec_memory_size {
                        min_spec_samples.pop_front();
                    }
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

    let (total_time_max, total_steps_sum) = {
        let mut total_time_max_result = 0_u64;
        let mut total_steps_sum_result = 0_u64;

        total_time_max.copy_to(&mut total_time_max_result)?;
        total_steps_sum.copy_to(&mut total_steps_sum_result)?;

        (
            f64::from_bits(total_time_max_result),
            total_steps_sum_result,
        )
    };

    Ok((total_time_max, total_steps_sum))
}

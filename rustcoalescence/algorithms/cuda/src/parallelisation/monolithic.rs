use std::{
    collections::VecDeque,
    convert::TryInto,
    num::{NonZeroU32, NonZeroU64},
};

use anyhow::{Context, Result};

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
        SingularActiveLineageSampler, SpeciationProbability, TurnoverRate,
    },
    event::{PackedEvent, TypedEvent},
    lineage::Lineage,
    reporter::{used::Unused, Reporter},
    simulation::Simulation,
};

use necsim_impls_no_std::{
    parallelisation::independent::{monolithic::WaterLevelReporter, DedupCache},
    partitioning::LocalPartition,
};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use crate::kernel::SimulationKernel;

#[allow(clippy::type_complexity, clippy::too_many_lines)]
pub fn simulate<
    'k,
    H: Habitat + RustToCuda,
    G: PrimeableRng + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: LineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, R, S> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    T: TurnoverRate<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry + RustToCuda,
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    P: Reporter,
    L: LocalPartition<P>,
>(
    mut simulation: Simulation<H, G, R, S, X, D, C, T, N, E, I, A>,
    kernel: &'k mut SimulationKernel<
        'k,
        H,
        G,
        R,
        S,
        X,
        D,
        C,
        T,
        N,
        E,
        I,
        A,
        <WaterLevelReporter<P> as Reporter>::ReportSpeciation,
        <WaterLevelReporter<P> as Reporter>::ReportDispersal,
    >,
    stream: &Stream,
    config: (GridSize, BlockSize, DedupCache, NonZeroU64),
    lineages: VecDeque<Lineage>,
    event_slice: NonZeroU32,
    local_partition: &mut L,
) -> Result<(f64, u64)> {
    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(lineages.len() as u64);

    let (grid_size, block_size, dedup_cache, step_slice) = config;

    // Allocate and initialise the total_time_max and total_steps_sum atomics
    let mut total_time_max = DeviceBox::new(&0.0_f64.to_bits())?;
    let mut total_steps_sum = DeviceBox::new(&0_u64)?;

    let mut task_list = ExchangeWithCudaWrapper::new(ValueBuffer::new(&block_size, &grid_size)?)?;
    let mut event_buffer: ExchangeWithCudaWrapper<
        EventBuffer<
            <WaterLevelReporter<P> as Reporter>::ReportSpeciation,
            <WaterLevelReporter<P> as Reporter>::ReportDispersal,
        >,
    > = ExchangeWithCudaWrapper::new(EventBuffer::new(
        &block_size,
        &grid_size,
        step_slice.get().try_into()?,
    )?)?;
    let mut min_spec_sample_buffer =
        ExchangeWithCudaWrapper::new(ValueBuffer::new(&block_size, &grid_size)?)?;

    let mut duplicate_individuals = bitvec::bitbox![0; min_spec_sample_buffer.len()];

    let mut kernel = kernel
        .with_dimensions(grid_size, block_size, 0_u32)
        .with_stream(stream);

    let mut min_spec_samples = dedup_cache.construct(lineages.len());

    let mut slow_lineages = lineages;
    let mut fast_lineages = VecDeque::new();

    let mut slow_events: Vec<PackedEvent> = Vec::with_capacity(event_slice.get() as usize);
    let mut fast_events: Vec<PackedEvent> = Vec::with_capacity(event_slice.get() as usize);

    let mut level_time = 0.0_f64;

    let cpu_habitat = simulation.habitat().backup();
    let cpu_turnover_rate = simulation.turnover_rate().backup();

    // TODO: We should use async launches and callbacks to rotate between
    // simulation, event analysis etc.
    simulation
        .lend_to_cuda_mut(|mut simulation_cuda_repr| {
            while !slow_lineages.is_empty() {
                let total_event_rate: f64 = slow_lineages
                    .iter()
                    .map(|lineage| {
                        cpu_turnover_rate.get_turnover_rate_at_location(
                            unsafe { lineage.indexed_location().unwrap_unchecked() }.location(),
                            &cpu_habitat,
                        )
                    })
                    .sum();
                level_time += f64::from(event_slice.get()) / total_event_rate;

                slow_events.extend(fast_events.drain_filter(|event| event.event_time < level_time));

                let mut reporter: WaterLevelReporter<P> =
                    WaterLevelReporter::new(level_time, &mut slow_events, &mut fast_events);

                while !slow_lineages.is_empty() {
                    // Upload the new tasks from the front of the task queue
                    for task in task_list.iter_mut() {
                        *task = slow_lineages.pop_front();
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
                            step_slice.get(),
                        )?;
                    }

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
                            if task.is_active() && !duplicate_individuals[i] {
                                if task.last_event_time() < level_time {
                                    slow_lineages.push_back(task);
                                } else {
                                    fast_lineages.push_back(task);
                                }
                            }
                        }
                    }

                    event_buffer.report_events(&mut reporter);

                    local_partition.get_reporter().report_progress(Unused::new(
                        &((slow_lineages.len() as u64) + (fast_lineages.len() as u64)),
                    ));
                }

                slow_events.sort();
                for event in slow_events.drain(..) {
                    match event.into() {
                        TypedEvent::Speciation(event) => {
                            local_partition
                                .get_reporter()
                                .report_speciation(Unused::new(&event));
                        },
                        TypedEvent::Dispersal(event) => {
                            local_partition
                                .get_reporter()
                                .report_dispersal(Unused::new(&event));
                        },
                    }
                }

                core::mem::swap(&mut slow_lineages, &mut fast_lineages);
            }

            Ok(())
        })
        .with_context(|| "Running the CUDA kernel failed.")?;

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

    local_partition.report_progress_sync(0_u64);

    Ok(local_partition.reduce_global_time_steps(total_time_max, total_steps_sum))
}

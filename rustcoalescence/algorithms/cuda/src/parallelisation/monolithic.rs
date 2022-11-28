use std::{
    collections::VecDeque,
    convert::{TryFrom, TryInto},
    num::NonZeroU64,
    sync::atomic::AtomicU64,
};

use rust_cuda::{
    common::RustToCuda,
    host::{HostAndDeviceMutRef, LendToCuda},
    rustacuda::{
        function::{BlockSize, GridSize},
        stream::Stream,
    },
    utils::exchange::wrapper::ExchangeWrapperOnHost,
};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
    reporter::{boolean::Boolean, Reporter},
    simulation::Simulation,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::{
    cogs::{
        active_lineage_sampler::singular::SingularActiveLineageSampler,
        event_sampler::tracking::MinSpeciationTrackingEventSampler,
    },
    parallelisation::{
        independent::{
            monolithic::reporter::{
                WaterLevelReporterConstructor, WaterLevelReporterProxy, WaterLevelReporterStrategy,
            },
            DedupCache, EventSlice,
        },
        Status,
    },
};
use necsim_partitioning_core::LocalPartition;

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rustcoalescence_algorithms_cuda_cpu_kernel::{SimulationKernel, SortKernel};
use rustcoalescence_algorithms_cuda_gpu_kernel::{SimulatableKernel, SortableKernel};

use crate::error::CudaError;

type Result<T, E = CudaError> = std::result::Result<T, E>;

#[allow(
    clippy::type_complexity,
    clippy::too_many_lines,
    clippy::too_many_arguments
)]
pub fn simulate<
    'l,
    'p,
    'stream,
    M: MathsCore,
    H: Habitat<M> + RustToCuda,
    G: Rng<M, Generator: PrimeableRng> + RustToCuda,
    S: LineageStore<M, H> + RustToCuda,
    X: EmigrationExit<M, H, G, S> + RustToCuda,
    D: DispersalSampler<M, H, G> + RustToCuda,
    C: CoalescenceSampler<M, H, S> + RustToCuda,
    T: TurnoverRate<M, H> + RustToCuda,
    N: SpeciationProbability<M, H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<M, H, G, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry<M> + RustToCuda,
    A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I>
        + RustToCuda,
    P: Reporter,
    L: LocalPartition<'p, P>,
    LI: IntoIterator<Item = Lineage>,
>(
    simulation: &mut Simulation<M, H, G, S, X, D, C, T, N, E, I, A>,
    mut kernel: SimulationKernel<
        M,
        H,
        G,
        S,
        X,
        D,
        C,
        T,
        N,
        E,
        I,
        A,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >,
    mut sort_kernel: SortKernel<
    <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
    <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >,
    config: (GridSize, BlockSize, DedupCache, NonZeroU64),
    stream: &'stream Stream,
    lineages: LI,
    event_slice: EventSlice,
    pause_before: Option<NonNegativeF64>,
    local_partition: &'l mut L,
) -> Result<(Status, NonNegativeF64, u64, impl IntoIterator<Item = Lineage>)>
    where SimulationKernel<
        M,
        H,
        G,
        S,
        X,
        D,
        C,
        T,
        N,
        E,
        I,
        A,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, 'p, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, 'p, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >: SimulatableKernel<
        M,
        H,
        G,
        S,
        X,
        D,
        C,
        T,
        N,
        E,
        I,
        A,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, 'p, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, 'p, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >,
{
    let mut slow_lineages = lineages
        .into_iter()
        .map(|lineage| {
            // We only need a strict lower bound here,
            //  i.e. that the next event >= pessimistic_next_event_time
            let pessimistic_next_event_time = lineage.last_event_time;

            (lineage, pessimistic_next_event_time)
        })
        .collect::<VecDeque<_>>();
    let mut fast_lineages = VecDeque::new();

    // Ensure that the progress bar starts with the expected target
    local_partition.report_progress_sync(slow_lineages.len() as u64);

    let event_slice = event_slice.capacity(slow_lineages.len());

    let mut proxy = <WaterLevelReporterStrategy as WaterLevelReporterConstructor<
        L::IsLive,
        P,
        L,
    >>::WaterLevelReporter::new(event_slice.get(), local_partition);

    let (grid_size, block_size, dedup_cache, step_slice) = config;

    #[allow(clippy::or_fun_call)]
    let intial_max_time = slow_lineages
        .iter()
        .map(|(lineage, _)| lineage.last_event_time)
        .max()
        .unwrap_or(NonNegativeF64::zero());

    // Initialise the total_time_max and total_steps_sum atomics
    let mut total_time_max = AtomicU64::new(intial_max_time.get().to_bits()).into();
    let mut total_steps_sum = AtomicU64::new(0_u64).into();

    let mut task_list = ExchangeWrapperOnHost::new(ValueBuffer::new(&block_size, &grid_size)?)?;
    let mut event_buffer: ExchangeWrapperOnHost<
        EventBuffer<
            <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<
                L::IsLive,
                P,
                L,
            >>::WaterLevelReporter as Reporter>::ReportSpeciation,
            <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<
                L::IsLive,
                P,
                L,
            >>::WaterLevelReporter as Reporter>::ReportDispersal,
        >,
    > = ExchangeWrapperOnHost::new(EventBuffer::new(
        &block_size,
        &grid_size,
        step_slice.get().try_into().unwrap_or(usize::MAX),
    )?)?;
    let mut min_spec_sample_buffer =
        ExchangeWrapperOnHost::new(ValueBuffer::new(&block_size, &grid_size)?)?;
    let mut next_event_time_buffer =
        ExchangeWrapperOnHost::new(ValueBuffer::new(&block_size, &grid_size)?)?;

    let max_events_per_type_individual = event_buffer.max_events_per_type_individual();

    let mut min_spec_samples = dedup_cache.construct(slow_lineages.len());

    #[allow(clippy::or_fun_call)]
    let mut level_time = slow_lineages
        .iter()
        .map(|(lineage, _)| lineage.last_event_time)
        .min()
        .unwrap_or(NonNegativeF64::zero());

    let cpu_habitat = simulation.habitat().backup();
    let cpu_turnover_rate = simulation.turnover_rate().backup();
    let cpu_speciation_probability = simulation.speciation_probability().backup();

    HostAndDeviceMutRef::with_new(&mut total_time_max, |total_time_max| -> Result<()> {
        HostAndDeviceMutRef::with_new(&mut total_steps_sum, |total_steps_sum| -> Result<()> {
            simulation.lend_to_cuda_mut(|mut simulation_cuda_repr| -> Result<()> {
                // Move the event buffer and min speciation sample buffer to CUDA
                let mut event_buffer_cuda = event_buffer.move_to_device_async(stream)?;
                let mut min_spec_sample_buffer_cuda =
                    min_spec_sample_buffer.move_to_device_async(stream)?;
                let mut next_event_time_buffer_cuda =
                    next_event_time_buffer.move_to_device_async(stream)?;

                while !slow_lineages.is_empty()
                    && pause_before.map_or(true, |pause_before| level_time < pause_before)
                {
                    let total_event_rate: NonNegativeF64 = if P::ReportDispersal::VALUE {
                        // Full event rate lambda with speciation
                        slow_lineages
                            .iter()
                            .map(|(lineage, _)| {
                                cpu_turnover_rate.get_turnover_rate_at_location(
                                    lineage.indexed_location.location(),
                                    &cpu_habitat,
                                )
                            })
                            .sum()
                    } else if P::ReportSpeciation::VALUE {
                        // Only speciation event rate lambda * nu
                        slow_lineages
                            .iter()
                            .map(|(lineage, _)| {
                                let location = lineage.indexed_location.location();

                                cpu_turnover_rate
                                    .get_turnover_rate_at_location(location, &cpu_habitat)
                                    * cpu_speciation_probability
                                        .get_speciation_probability_at_location(
                                            location,
                                            &cpu_habitat,
                                        )
                            })
                            .sum()
                    } else {
                        // No events produced -> no restriction
                        NonNegativeF64::zero()
                    };

                    level_time += NonNegativeF64::from(event_slice.get()) / total_event_rate;

                    if let Some(pause_before) = pause_before {
                        level_time = level_time.min(pause_before);
                    }

                    // [Report all events below the water level] + Advance the water level
                    proxy.advance_water_level(level_time);

                    // Simulate all slow lineages until they have finished or exceeded the
                    // new water  level
                    while !slow_lineages.is_empty() {
                        let mut num_tasks = 0_usize;

                        // Upload the new tasks from the front of the task queue
                        for mut task in task_list.iter_mut() {
                            let next_slow_lineage = loop {
                                match slow_lineages.pop_front() {
                                    None => break None,
                                    Some((slow_lineage, next_event)) if next_event < level_time => {
                                        break Some(slow_lineage)
                                    },
                                    Some((fast_lineage, next_event)) => {
                                        fast_lineages.push_back((fast_lineage, next_event));
                                    },
                                }
                            };

                            task.replace(next_slow_lineage);
                            num_tasks += 1;
                        }

                        // Move the task list, event buffer and min speciation sample buffer
                        // to CUDA
                        let mut task_list_cuda = task_list.move_to_device_async(stream)?;

                        // TODO: Investigate distributing over several streams
                        kernel.simulate_async(
                            stream,
                            simulation_cuda_repr.as_mut().as_async(),
                            task_list_cuda.as_mut_async(),
                            event_buffer_cuda.as_mut_async(),
                            min_spec_sample_buffer_cuda.as_mut_async(),
                            next_event_time_buffer_cuda.as_mut_async(),
                            total_time_max.as_ref().as_async(),
                            total_steps_sum.as_ref().as_async(),
                            step_slice.get().into(),
                            level_time.into(),
                        )?;

                        let min_spec_sample_buffer_host =
                            min_spec_sample_buffer_cuda.move_to_host_async(stream)?;
                        let next_event_time_buffer_host =
                            next_event_time_buffer_cuda.move_to_host_async(stream)?;
                        let task_list_host = task_list_cuda.move_to_host_async(stream)?;

                        let mut size = 2;

                        while size <= num_tasks * max_events_per_type_individual {
                            let mut stride = size / 2;

                            while stride > 0 {
                                let grid = u32::try_from(
                                    num_tasks * max_events_per_type_individual / 512,
                                )
                                .map_err(|_| {
                                    rust_cuda::rustacuda::error::CudaError::LaunchOutOfResources
                                })?;

                                sort_kernel.with_grid(grid.into()).sort_events_async(
                                    stream,
                                    event_buffer_cuda.as_mut_async(),
                                    size.into(),
                                    stride.into(),
                                )?;

                                stride >>= 1;
                            }

                            size <<= 1;
                        }

                        let event_buffer_host = event_buffer_cuda.move_to_host_async(stream)?;

                        min_spec_sample_buffer = min_spec_sample_buffer_host.sync_to_host()?;
                        next_event_time_buffer = next_event_time_buffer_host.sync_to_host()?;
                        task_list = task_list_host.sync_to_host()?;

                        // Fetch the completion of the tasks
                        for ((mut spec_sample, mut next_event_time), mut task) in
                            min_spec_sample_buffer
                                .iter_mut()
                                .zip(next_event_time_buffer.iter_mut())
                                .zip(task_list.iter_mut())
                        {
                            let duplicate_individual = spec_sample
                                .take()
                                .map_or(false, |spec_sample| !min_spec_samples.insert(spec_sample));

                            if let (Some(task), Some(next_event_time)) =
                                (task.take(), next_event_time.take())
                            {
                                if !duplicate_individual {
                                    // Reclassify lineages as either slow (still below
                                    // water) or
                                    // fast
                                    if next_event_time < level_time {
                                        slow_lineages.push_back((task, next_event_time.into()));
                                    } else {
                                        fast_lineages.push_back((task, next_event_time.into()));
                                    }
                                }
                            }
                        }

                        min_spec_sample_buffer_cuda =
                            min_spec_sample_buffer.move_to_device_async(stream)?;
                        next_event_time_buffer_cuda =
                            next_event_time_buffer.move_to_device_async(stream)?;

                        event_buffer = event_buffer_host.sync_to_host()?;
                        // event_buffer.sort_events();
                        event_buffer.report_events_unordered(&mut proxy);
                        event_buffer_cuda = event_buffer.move_to_device_async(stream)?;

                        proxy.local_partition().get_reporter().report_progress(
                            &((slow_lineages.len() as u64) + (fast_lineages.len() as u64)).into(),
                        );
                    }

                    // Fast lineages are now slow again
                    std::mem::swap(&mut slow_lineages, &mut fast_lineages);
                }

                Ok(())
            })?;

            // [Report all remaining events]
            proxy.finalise();

            Ok(())
        })
    })?;

    // Safety: Max of NonNegativeF64 values from the GPU
    let total_time_max = unsafe {
        NonNegativeF64::new_unchecked(f64::from_bits(total_time_max.into_inner().into_inner()))
    };
    let total_steps_sum = total_steps_sum.into_inner().into_inner();

    local_partition.report_progress_sync(slow_lineages.len() as u64);

    let status = Status::paused(local_partition.reduce_vote_continue(!slow_lineages.is_empty()));
    let (global_time, global_steps) =
        local_partition.reduce_global_time_steps(total_time_max, total_steps_sum);
    let lineages = slow_lineages.into_iter().map(|(lineage, _)| lineage);

    Ok((status, global_time, global_steps, lineages))
}

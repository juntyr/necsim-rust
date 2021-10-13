use std::{collections::VecDeque, convert::TryInto, num::NonZeroU64, sync::atomic::AtomicU64};

use anyhow::{Context, Result};

use rust_cuda::{
    host::HostAndDeviceMutRef,
    rustacuda::function::{BlockSize, GridSize},
};

use rust_cuda::{
    common::RustToCuda, host::LendToCuda, utils::exchange::wrapper::ExchangeWrapperOnHost,
};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, PrimeableRng, SpeciationProbability, TurnoverRate,
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
    parallelisation::independent::{
        monolithic::reporter::{
            WaterLevelReporterConstructor, WaterLevelReporterProxy, WaterLevelReporterStrategy,
        },
        DedupCache, EventSlice,
    },
};
use necsim_partitioning_core::LocalPartition;

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rustcoalescence_algorithms_cuda_kernel::Kernel;

use crate::kernel::SimulationKernel;

#[allow(clippy::type_complexity, clippy::too_many_lines)]
pub fn simulate<
    'l,
    F: F64Core,
    H: Habitat<F> + RustToCuda,
    G: PrimeableRng<F> + RustToCuda,
    R: LineageReference<F, H>,
    S: LineageStore<F, H, R> + RustToCuda,
    X: EmigrationExit<F, H, G, R, S> + RustToCuda,
    D: DispersalSampler<F, H, G> + RustToCuda,
    C: CoalescenceSampler<F, H, R, S> + RustToCuda,
    T: TurnoverRate<F, H> + RustToCuda,
    N: SpeciationProbability<F, H> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<F, H, G, R, S, X, D, C, T, N> + RustToCuda,
    I: ImmigrationEntry<F> + RustToCuda,
    A: SingularActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>
        + RustToCuda,
    P: Reporter,
    L: LocalPartition<P>,
    LI: IntoIterator<Item = Lineage>,
>(
    mut simulation: Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A>,
    mut kernel: SimulationKernel<
        F,
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
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >,
    config: (GridSize, BlockSize, DedupCache, NonZeroU64),
    lineages: LI,
    event_slice: EventSlice,
    local_partition: &'l mut L,
) -> Result<(NonNegativeF64, u64)>
    where SimulationKernel<
        F,
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
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
    >: rustcoalescence_algorithms_cuda_kernel::Kernel<
        F,
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
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportSpeciation,
        <<WaterLevelReporterStrategy as WaterLevelReporterConstructor<'l, L::IsLive, P, L>>::WaterLevelReporter as Reporter>::ReportDispersal,
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

    // Initialise the total_time_max and total_steps_sum atomics
    let mut total_time_max = AtomicU64::new(0.0_f64.to_bits()).into();
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
        step_slice.get().try_into()?,
    )?)?;
    let mut min_spec_sample_buffer =
        ExchangeWrapperOnHost::new(ValueBuffer::new(&block_size, &grid_size)?)?;
    let mut next_event_time_buffer =
        ExchangeWrapperOnHost::new(ValueBuffer::new(&block_size, &grid_size)?)?;

    let mut min_spec_samples = dedup_cache.construct(slow_lineages.len());

    let mut level_time = NonNegativeF64::zero();

    let cpu_habitat = simulation.habitat().backup();
    let cpu_turnover_rate = simulation.turnover_rate().backup();
    let cpu_speciation_probability = simulation.speciation_probability().backup();

    HostAndDeviceMutRef::with_new(&mut total_time_max, |total_time_max| -> Result<()> {
        HostAndDeviceMutRef::with_new(&mut total_steps_sum, |total_steps_sum| -> Result<()> {
            // TODO: Pipeline async launches and callbacks of simulation/event analysis
            simulation
                .lend_to_cuda_mut(|mut simulation_cuda_repr| -> Result<()> {
                    while !slow_lineages.is_empty() {
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

                        // [Report all events below the water level] + Advance the water level
                        proxy.advance_water_level(level_time);

                        // Simulate all slow lineages until they have finished or exceeded the new
                        // water  level
                        while !slow_lineages.is_empty() {
                            // Upload the new tasks from the front of the task queue
                            for mut task in task_list.iter_mut() {
                                let next_slow_lineage = loop {
                                    match slow_lineages.pop_front() {
                                        None => break None,
                                        Some((slow_lineage, next_event))
                                            if next_event < level_time =>
                                        {
                                            break Some(slow_lineage)
                                        },
                                        Some((fast_lineage, next_event)) => {
                                            fast_lineages.push_back((fast_lineage, next_event));
                                        },
                                    }
                                };

                                task.replace(next_slow_lineage);
                            }

                            // Move the task list, event buffer and min speciation sample buffer to
                            // CUDA
                            let mut event_buffer_cuda = event_buffer.move_to_device()?;
                            let mut min_spec_sample_buffer_cuda =
                                min_spec_sample_buffer.move_to_device()?;
                            let mut next_event_time_buffer_cuda =
                                next_event_time_buffer.move_to_device()?;
                            let mut task_list_cuda = task_list.move_to_device()?;

                            kernel.simulate_raw(
                                simulation_cuda_repr.as_mut(),
                                task_list_cuda.as_mut(),
                                event_buffer_cuda.as_mut(),
                                min_spec_sample_buffer_cuda.as_mut(),
                                next_event_time_buffer_cuda.as_mut(),
                                total_time_max.as_ref(),
                                total_steps_sum.as_ref(),
                                step_slice.get().into(),
                                level_time.into(),
                            )?;

                            min_spec_sample_buffer = min_spec_sample_buffer_cuda.move_to_host()?;
                            next_event_time_buffer = next_event_time_buffer_cuda.move_to_host()?;
                            task_list = task_list_cuda.move_to_host()?;
                            event_buffer = event_buffer_cuda.move_to_host()?;

                            // Fetch the completion of the tasks
                            for ((mut spec_sample, mut next_event_time), mut task) in
                                min_spec_sample_buffer
                                    .iter_mut()
                                    .zip(next_event_time_buffer.iter_mut())
                                    .zip(task_list.iter_mut())
                            {
                                let duplicate_individual =
                                    spec_sample.take().map_or(false, |spec_sample| {
                                        !min_spec_samples.insert(spec_sample)
                                    });

                                if let (Some(task), Some(next_event_time)) =
                                    (task.take(), next_event_time.take())
                                {
                                    if !duplicate_individual {
                                        // Reclassify lineages as either slow (still below water) or
                                        // fast
                                        if next_event_time < level_time {
                                            slow_lineages.push_back((task, next_event_time.into()));
                                        } else {
                                            fast_lineages.push_back((task, next_event_time.into()));
                                        }
                                    }
                                }
                            }

                            event_buffer.report_events(&mut proxy);

                            proxy.local_partition().get_reporter().report_progress(
                                &((slow_lineages.len() as u64) + (fast_lineages.len() as u64))
                                    .into(),
                            );
                        }

                        // Fast lineages are now slow again
                        std::mem::swap(&mut slow_lineages, &mut fast_lineages);
                    }

                    Ok(())
                })
                .with_context(|| "Running the CUDA kernel failed.")?;

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

    local_partition.report_progress_sync(0_u64);

    Ok(local_partition.reduce_global_time_steps(total_time_max, total_steps_sum))
}

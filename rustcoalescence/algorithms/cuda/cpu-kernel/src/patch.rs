use std::sync::atomic::AtomicU64;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
    reporter::boolean::{Boolean, False, True},
    simulation::Simulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::{MinSpeciationTrackingEventSampler, SpeciationSample},
};

use rust_cuda::{
    common::{DeviceAccessible, RustToCuda},
    host::{HostAndDeviceConstRefAsync, HostAndDeviceMutRefAsync, TypedKernel},
    rustacuda::{error::CudaResult, stream::Stream},
    utils::device_copy::SafeDeviceCopyWrapper,
};

use rustcoalescence_algorithms_cuda_gpu_kernel::{SimulatableKernel, SortableKernel};

use crate::{SimulationKernel, SortKernel};

// If `Kernel` is implemented for `ReportSpeciation` x `ReportDispersal`, i.e.
//  for {`False`, `True`} x {`False`, `True`} then it is implemented for all
//  `Boolean`s. However, Rust does not recognise that `Boolean` is closed over
//  {`False`, `True`}. These default impls provide the necessary coersion.

extern "C" {
    fn unreachable_cuda_simulation_linking_reporter() -> !;
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<
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
        A: SingularActiveLineageSampler<M, H, G, S, X, D, C, T, N, E, I> + RustToCuda,
        ReportSpeciation: Boolean,
        ReportDispersal: Boolean,
    > SimulatableKernel<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
    for SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, ReportSpeciation, ReportDispersal>
where
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, False>:
        SimulatableKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, False>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, True>:
        SimulatableKernel<M, H, G, S, X, D, C, T, N, E, I, A, False, True>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, False>:
        SimulatableKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, False>,
    SimulationKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, True>:
        SimulatableKernel<M, H, G, S, X, D, C, T, N, E, I, A, True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel() -> CudaResult<
        TypedKernel<
            dyn SimulatableKernel<
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
                ReportSpeciation,
                ReportDispersal,
            >,
        >,
    > {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn simulate<'stream>(
        &mut self,
        _stream: &'stream Stream,
        _simulation: &mut Simulation<M, H, G, S, X, D, C, T, N, E, I, A>,
        _task_list: &mut ValueBuffer<Lineage, true, true>,
        _event_buffer_reporter: &mut EventBuffer<ReportSpeciation, ReportDispersal>,
        _min_spec_sample_buffer: &mut ValueBuffer<SpeciationSample, false, true>,
        _next_event_time_buffer: &mut ValueBuffer<PositiveF64, false, true>,
        _total_time_max: &AtomicU64,
        _total_steps_sum: &AtomicU64,
        _max_steps: u64,
        _max_next_event_time: NonNegativeF64,
    ) -> CudaResult<()> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn simulate_async<'stream>(
        &mut self,
        _stream: &'stream Stream,
        _simulation: HostAndDeviceMutRefAsync<
            DeviceAccessible<
                <Simulation<M, H, G, S, X, D, C, T, N, E, I, A> as RustToCuda>::CudaRepresentation,
            >,
        >,
        _task_list: HostAndDeviceMutRefAsync<
            DeviceAccessible<<ValueBuffer<Lineage, true, true> as RustToCuda>::CudaRepresentation>,
        >,
        _event_buffer_reporter: HostAndDeviceMutRefAsync<
            DeviceAccessible<
                <EventBuffer<ReportSpeciation, ReportDispersal> as RustToCuda>::CudaRepresentation,
            >,
        >,
        _min_spec_sample_buffer: HostAndDeviceMutRefAsync<
            DeviceAccessible<
                <ValueBuffer<SpeciationSample, false, true> as RustToCuda>::CudaRepresentation,
            >,
        >,
        _next_event_time_buffer: HostAndDeviceMutRefAsync<
            DeviceAccessible<
                <ValueBuffer<PositiveF64, false, true> as RustToCuda>::CudaRepresentation,
            >,
        >,
        _total_time_max: HostAndDeviceConstRefAsync<SafeDeviceCopyWrapper<AtomicU64>>,
        _total_steps_sum: HostAndDeviceConstRefAsync<SafeDeviceCopyWrapper<AtomicU64>>,
        _max_steps: SafeDeviceCopyWrapper<u64>,
        _max_next_event_time: SafeDeviceCopyWrapper<NonNegativeF64>,
    ) -> CudaResult<()> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
unsafe impl<ReportSpeciation: Boolean, ReportDispersal: Boolean>
    SortableKernel<ReportSpeciation, ReportDispersal>
    for SortKernel<ReportSpeciation, ReportDispersal>
where
    SortKernel<False, False>: SortableKernel<False, False>,
    SortKernel<False, True>: SortableKernel<False, True>,
    SortKernel<True, False>: SortableKernel<True, False>,
    SortKernel<True, True>: SortableKernel<True, True>,
{
    default fn get_ptx_str() -> &'static str {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn new_kernel(
    ) -> CudaResult<TypedKernel<dyn SortableKernel<ReportSpeciation, ReportDispersal>>> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn sort_events<'stream>(
        &mut self,
        _stream: &'stream Stream,
        _event_buffer_reporter: &mut EventBuffer<ReportSpeciation, ReportDispersal>,
        _size: usize,
        _stride: usize,
    ) -> CudaResult<()> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }

    default fn sort_events_async<'stream>(
        &mut self,
        _stream: &'stream Stream,
        _event_buffer_reporter: HostAndDeviceMutRefAsync<
            DeviceAccessible<
                <EventBuffer<ReportSpeciation, ReportDispersal> as RustToCuda>::CudaRepresentation,
            >,
        >,
        _size: SafeDeviceCopyWrapper<usize>,
        _stride: SafeDeviceCopyWrapper<usize>,
    ) -> CudaResult<()> {
        unsafe { unreachable_cuda_simulation_linking_reporter() }
    }
}

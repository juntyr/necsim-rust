#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(atomic_from_mut)]
#![feature(asm)]

extern crate alloc;

#[macro_use]
extern crate rustcoalescence_algorithms_cuda_kernel_specialiser;

use core::sync::atomic::{AtomicU64, Ordering};

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, MinSpeciationTrackingEventSampler,
        PeekableActiveLineageSampler, PrimeableRng, SingularActiveLineageSampler,
        SpeciationProbability, SpeciationSample, TurnoverRate,
    },
    lineage::Lineage,
    reporter::boolean::Boolean,
    simulation::Simulation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use rustacuda_core::DeviceCopy;

use rustcoalescence_algorithms_cuda_kernel_ptx_jit::PtxJITConstLoad;

use rust_cuda::{
    common::{DeviceBoxMut, RustToCuda},
    device::{nvptx, utils, AnyDeviceBoxMut, BorrowFromRust},
};

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[cfg(not(debug_assertions))]
#[panic_handler]
fn panic(_panic_info: &::core::panic::PanicInfo) -> ! {
    unsafe { nvptx::trap() }
}

#[cfg(debug_assertions)]
#[panic_handler]
fn panic(panic_info: &::core::panic::PanicInfo) -> ! {
    use rust_cuda::println;

    println!(
        "Panic occurred at {:?}: {:?}!",
        panic_info.location(),
        panic_info
            .message()
            .unwrap_or(&format_args!("unknown reason"))
    );

    unsafe { nvptx::trap() }
}

#[alloc_error_handler]
fn alloc_error_handler(_: core::alloc::Layout) -> ! {
    unsafe { nvptx::trap() }
}

/// # Safety
/// This CUDA kernel is unsafe as it is called with untyped `AnyDeviceBox`.
#[no_mangle]
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_any: AnyDeviceBoxMut,
    task_list_any: AnyDeviceBoxMut,
    event_buffer_any: AnyDeviceBoxMut,
    min_spec_sample_buffer_any: AnyDeviceBoxMut,
    next_event_time_buffer_any: AnyDeviceBoxMut,
    total_time_max: AnyDeviceBoxMut,
    total_steps_sum: AnyDeviceBoxMut,
    max_steps: u64,
    max_next_event_time: NonNegativeF64,
) {
    specialise!(simulate_generic)(
        simulation_any.into(),
        task_list_any.into(),
        event_buffer_any.into(),
        min_spec_sample_buffer_any.into(),
        next_event_time_buffer_any.into(),
        total_time_max.into(),
        total_steps_sum.into(),
        max_steps,
        max_next_event_time,
    )
}

#[inline]
unsafe fn simulate_generic<
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
    A: SingularActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
        + PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>
        + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
>(
    simulation_cuda_repr: DeviceBoxMut<
        <Simulation<H, G, R, S, X, D, C, T, N, E, I, A> as RustToCuda>::CudaRepresentation,
    >,
    task_list_cuda_repr: DeviceBoxMut<<ValueBuffer<Lineage> as RustToCuda>::CudaRepresentation>,
    event_buffer_cuda_repr: DeviceBoxMut<
        <EventBuffer<ReportSpeciation, ReportDispersal> as RustToCuda>::CudaRepresentation,
    >,
    min_spec_sample_buffer_cuda_repr: DeviceBoxMut<
        <ValueBuffer<SpeciationSample> as RustToCuda>::CudaRepresentation,
    >,
    next_event_time_buffer_cuda_repr: DeviceBoxMut<
        <ValueBuffer<PositiveF64> as RustToCuda>::CudaRepresentation,
    >,
    mut total_time_max: DeviceBoxMut<u64>,
    mut total_steps_sum: DeviceBoxMut<u64>,
    max_steps: u64,
    max_next_event_time: NonNegativeF64,
) {
    PtxJITConstLoad!([0] => simulation_cuda_repr.as_ref());
    PtxJITConstLoad!([1] => task_list_cuda_repr.as_ref());
    PtxJITConstLoad!([2] => event_buffer_cuda_repr.as_ref());
    PtxJITConstLoad!([3] => min_spec_sample_buffer_cuda_repr.as_ref());
    PtxJITConstLoad!([4] => next_event_time_buffer_cuda_repr.as_ref());

    let total_time_max = AtomicU64::from_mut(total_time_max.as_mut());
    let total_steps_sum = AtomicU64::from_mut(total_steps_sum.as_mut());

    Simulation::with_borrow_from_rust_mut(simulation_cuda_repr, |simulation| {
        ValueBuffer::with_borrow_from_rust_mut(task_list_cuda_repr, |task_list| {
            task_list.with_value_for_core(|task| {
                // Discard the prior task (the simulation is just a temporary local copy)
                core::mem::drop(
                    simulation
                        .active_lineage_sampler_mut()
                        .replace_active_lineage(task),
                );

                EventBuffer::with_borrow_from_rust_mut(
                    event_buffer_cuda_repr,
                    |event_buffer_reporter| {
                        ValueBuffer::with_borrow_from_rust_mut(
                            min_spec_sample_buffer_cuda_repr,
                            |min_spec_sample_buffer| {
                                min_spec_sample_buffer.with_value_for_core(|min_spec_sample| {
                                    // Discard the prior sample (same reason as above)
                                    simulation
                                        .event_sampler_mut()
                                        .replace_min_speciation(min_spec_sample);

                                    let (time, steps) = simulation.simulate_incremental_early_stop(
                                        |simulation, steps| {
                                            steps >= max_steps
                                                || simulation
                                                    .peek_time_of_next_event()
                                                    .map_or(true, |next_time| {
                                                        next_time >= max_next_event_time
                                                    })
                                        },
                                        event_buffer_reporter,
                                    );

                                    ValueBuffer::with_borrow_from_rust_mut(
                                        next_event_time_buffer_cuda_repr,
                                        |next_event_time_buffer| {
                                            next_event_time_buffer.with_value_for_core(|_| {
                                                simulation.peek_time_of_next_event()
                                            })
                                        },
                                    );

                                    if steps > 0 {
                                        total_time_max
                                            .fetch_max(time.get().to_bits(), Ordering::Relaxed);
                                        total_steps_sum.fetch_add(steps, Ordering::Relaxed);
                                    }

                                    simulation.event_sampler_mut().replace_min_speciation(None)
                                })
                            },
                        )
                    },
                );

                simulation
                    .active_lineage_sampler_mut()
                    .replace_active_lineage(None)
            })
        })
    })
}

#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(min_const_generics)]

extern crate alloc;

#[macro_use]
extern crate specialiser;

use rust_cuda::device::{nvptx, utils};

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

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, IncoherentLineageStore,
        LineageReference, MinSpeciationTrackingEventSampler, PrimeableRng,
        SingularActiveLineageSampler, SpeciationProbability, SpeciationSample,
    },
    simulation::Simulation,
};
use rust_cuda::{common::RustToCuda, device::BorrowFromRust};
use rustacuda_core::DeviceCopy;

use necsim_impls_cuda::{event_buffer::EventBuffer, value_buffer::ValueBuffer};

use core::sync::atomic::{AtomicU64, Ordering};

extern "C" {
    static global_time_max: AtomicU64;
    static global_steps_sum: AtomicU64;
}

use rust_cuda::device::AnyDeviceBoxMut;

/// # Safety
/// This CUDA kernel is unsafe as it is called with untyped `AnyDeviceBox`.
#[no_mangle]
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_any: AnyDeviceBoxMut,
    task_list_any: AnyDeviceBoxMut,
    event_buffer_any: AnyDeviceBoxMut,
    min_spec_sample_buffer_any: AnyDeviceBoxMut,
    max_steps: u64,
) {
    specialise!(simulate_generic)(
        simulation_any.into(),
        task_list_any.into(),
        event_buffer_any.into(),
        min_spec_sample_buffer_any.into(),
        max_steps,
    )
}

use rust_cuda::common::DeviceBoxMut;

#[inline]
unsafe fn simulate_generic<
    H: Habitat + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    N: SpeciationProbability<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    X: EmigrationExit<H, G, N, D, R, S> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: MinSpeciationTrackingEventSampler<H, G, N, D, R, S, X, C> + RustToCuda,
    A: SingularActiveLineageSampler<H, G, N, D, R, S, X, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    simulation_cuda_repr: DeviceBoxMut<
        <Simulation<H, G, N, D, R, S, X, C, E, A> as RustToCuda>::CudaRepresentation,
    >,
    task_list_cuda_repr: DeviceBoxMut<<ValueBuffer<R> as RustToCuda>::CudaRepresentation>,
    event_buffer_cuda_repr: DeviceBoxMut<
        <EventBuffer<REPORT_SPECIATION, REPORT_DISPERSAL> as RustToCuda>::CudaRepresentation,
    >,
    min_spec_sample_buffer_cuda_repr: DeviceBoxMut<
        <ValueBuffer<SpeciationSample> as RustToCuda>::CudaRepresentation,
    >,
    max_steps: u64,
) {
    Simulation::with_borrow_from_rust_mut(simulation_cuda_repr, |simulation| {
        ValueBuffer::with_borrow_from_rust_mut(task_list_cuda_repr, |task_list| {
            task_list.with_value_for_core(|task| {
                let saved_task = simulation.with_mut_split_active_lineage_sampler_and_rng(
                    |active_lineage_sampler, simulation, _rng| {
                        active_lineage_sampler
                            .replace_active_lineage(task, &mut simulation.lineage_store)
                    },
                );

                EventBuffer::with_borrow_from_rust_mut(
                    event_buffer_cuda_repr,
                    |event_buffer_reporter| {
                        ValueBuffer::with_borrow_from_rust_mut(
                            min_spec_sample_buffer_cuda_repr,
                            |min_spec_sample_buffer| {
                                min_spec_sample_buffer.with_value_for_core(|min_spec_sample| {
                                    let old_min_spec_sample = simulation
                                        .event_sampler_mut()
                                        .replace_min_speciation(min_spec_sample);

                                    let (time, steps) = simulation
                                        .simulate_incremental(max_steps, event_buffer_reporter);

                                    if steps > 0 {
                                        global_time_max
                                            .fetch_max(time.to_bits(), Ordering::Relaxed);
                                        global_steps_sum.fetch_add(steps, Ordering::Relaxed);
                                    }

                                    simulation
                                        .event_sampler_mut()
                                        .replace_min_speciation(old_min_spec_sample)
                                })
                            },
                        )
                    },
                );

                simulation.with_mut_split_active_lineage_sampler_and_rng(
                    |active_lineage_sampler, simulation, _rng| {
                        active_lineage_sampler
                            .replace_active_lineage(saved_task, &mut simulation.lineage_store)
                    },
                )
            })
        })
    })
}

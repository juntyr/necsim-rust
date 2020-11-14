#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]
#![feature(min_const_generics)]

extern crate alloc;

use rust_cuda::{
    device::{nvptx, utils},
    println,
};

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[panic_handler]
fn panic(panic_info: &::core::panic::PanicInfo) -> ! {
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

struct F32(f32);
struct F64(f64);

impl core::fmt::Debug for F32 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

impl core::fmt::Debug for F64 {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{}", ryu::Buffer::new().format(self.0))
    }
}

use necsim_core::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
        HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
    },
    simulation::Simulation,
};
use rust_cuda::{common::RustToCuda, device::BorrowFromRust};
use rustacuda_core::DeviceCopy;

use necsim_impls_cuda::event_buffer::{
    common::EventBufferCudaRepresentation, device::EventBufferDevice,
};

use core::sync::atomic::{AtomicU64, Ordering};

extern "C" {
    #[no_mangle]
    static global_lineages_remaining: AtomicU64;
    #[no_mangle]
    static global_time_max: AtomicU64;
    #[no_mangle]
    static global_steps_sum: AtomicU64;
}

#[no_mangle]
/// # Safety
/// This CUDA kernel is unsafe as it is called with raw pointers
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_c_ptr: *mut core::ffi::c_void,
    event_buffer_c_ptr: *mut core::ffi::c_void,
    max_steps: u64,
) {
    simulate_generic(
        simulation_c_ptr as *mut <config::Simulation as RustToCuda>::CudaRepresentation,
        event_buffer_c_ptr as *mut config::EventBufferCudaRepresentation,
        max_steps,
    )
}

unsafe fn simulate_generic<
    H: HabitatToU64Injection + RustToCuda,
    G: PrimeableRng<H> + RustToCuda,
    D: DispersalSampler<H, G> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, G, R, S> + RustToCuda,
    E: EventSampler<H, G, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, G, D, R, S, C, E> + RustToCuda,
    const REPORT_SPECIATION: bool,
    const REPORT_DISPERSAL: bool,
>(
    simulation_ptr: *mut <Simulation<H, G, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
    event_buffer_ptr: *mut EventBufferCudaRepresentation<H, R, REPORT_SPECIATION, REPORT_DISPERSAL>,
    max_steps: u64,
) {
    Simulation::with_borrow_from_rust_mut(simulation_ptr, |simulation| {
        EventBufferDevice::with_borrow_from_rust_mut(event_buffer_ptr, |event_buffer_reporter| {
            let active_lineages_remaining_before =
                simulation.active_lineage_sampler().number_active_lineages();

            let (time, steps) = simulation.simulate_incremental(max_steps, event_buffer_reporter);

            let active_lineages_remaining_after =
                simulation.active_lineage_sampler().number_active_lineages();

            if steps > 0
                && active_lineages_remaining_after == 0
                && active_lineages_remaining_before > 0
            {
                global_lineages_remaining
                    .fetch_sub(active_lineages_remaining_before as u64, Ordering::Relaxed);

                global_time_max.fetch_max(time.to_bits(), Ordering::Relaxed);
                global_steps_sum.fetch_add(steps, Ordering::Relaxed);
            }
        })
    })
}

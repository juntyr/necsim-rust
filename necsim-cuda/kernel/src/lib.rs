#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]

extern crate alloc;

use rust_cuda::{
    device::{nvptx, utils},
    println,
};

#[global_allocator]
static _GLOBAL_ALLOCATOR: utils::PTXAllocator = utils::PTXAllocator;

#[panic_handler]
fn panic(_info: &::core::panic::PanicInfo) -> ! {
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

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
    IncoherentLineageStore, LineageReference,
};
use necsim_core::simulation::Simulation;
use rust_cuda::common::RustToCuda;
use rust_cuda::device::BorrowFromRust;

#[no_mangle]
/// # Safety
/// This CUDA kernel is unsafe as it is called with raw pointers
pub unsafe extern "ptx-kernel" fn simulate<
    H: Habitat + RustToCuda,
    D: DispersalSampler<H> + RustToCuda,
    R: LineageReference<H> + RustToCuda,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    E: EventSampler<H, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, D, R, S, C, E> + RustToCuda,
>(
    simulation_ptr: *const <Simulation<H, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
) {
    <Simulation<H, D, R, S, C, E, A> as BorrowFromRust>::with_borrow_from_rust(
        simulation_ptr,
        |simulation| {
            println!("Hello Simulation on CUDA!");
        },
    )
}

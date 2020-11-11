#![cfg(target_os = "cuda")]
#![deny(clippy::pedantic)]
#![no_std]
#![feature(abi_ptx)]
#![feature(alloc_error_handler)]
#![feature(panic_info_message)]

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

use necsim_core::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler,
    HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
};
use necsim_core::simulation::Simulation;
use rust_cuda::common::RustToCuda;
use rust_cuda::device::BorrowFromRust;
use rustacuda_core::DeviceCopy;

use necsim_impls_cuda::cogs::rng::CudaRng;
use necsim_impls_cuda::event_buffer::common::EventBufferCudaRepresentation;
use necsim_impls_cuda::event_buffer::device::EventBufferDevice;

#[no_mangle]
/// # Safety
/// This CUDA kernel is unsafe as it is called with raw pointers
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_c_ptr: *mut core::ffi::c_void,
    event_buffer_c_ptr: *mut core::ffi::c_void,
    max_steps: usize,
) {
    use necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler as ActiveLineageSampler;
    use necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler as CoalescenceSampler;
    use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler as DispersalSampler;
    use necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler as EventSampler;
    use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat as Habitat;
    use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference as LineageReference;
    use necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore as LineageStore;
    use necsim_impls_no_std::cogs::rng::wyhash::WyHash as Rng;

    simulate_generic(
        simulation_c_ptr
            as *mut <Simulation<
                Habitat,
                CudaRng<Rng>,
                DispersalSampler<_, _>,
                LineageReference,
                LineageStore<_>,
                CoalescenceSampler<_, _, _, _>,
                EventSampler<_, _, _, _, _>,
                ActiveLineageSampler<_, _, _, _, _>,
            > as RustToCuda>::CudaRepresentation,
        event_buffer_c_ptr as *mut EventBufferCudaRepresentation<Habitat, LineageReference>,
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
>(
    simulation_ptr: *mut <Simulation<H, G, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
    event_buffer_ptr: *mut EventBufferCudaRepresentation<H, R>,
    max_steps: usize,
) {
    Simulation::with_borrow_from_rust_mut(simulation_ptr, |simulation| {
        EventBufferDevice::with_borrow_from_rust_mut(event_buffer_ptr, |event_buffer_reporter| {
            simulation.simulate_incremental(max_steps, event_buffer_reporter);
        })
    })
}

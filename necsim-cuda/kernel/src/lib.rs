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
use necsim_core::reporter::NullReporter;
use necsim_core::rng::Rng;
use necsim_core::simulation::Simulation;
use necsim_impls_no_std::rng::cuda::CudaRng;
use rust_cuda::common::RustToCuda;
use rust_cuda::device::BorrowFromRust;
use rustacuda_core::DeviceCopy;

#[no_mangle]
/// # Safety
/// This CUDA kernel is unsafe as it is called with raw pointers
pub unsafe extern "ptx-kernel" fn simulate(
    simulation_c_ptr: *mut core::ffi::c_void,
    cuda_rng_c_ptr: *mut core::ffi::c_void,
    max_steps: usize,
) {
    use necsim_impls_no_std::cogs::active_lineage_sampler::independent::IndependentActiveLineageSampler as ActiveLineageSampler;
    use necsim_impls_no_std::cogs::coalescence_sampler::independent::IndependentCoalescenceSampler as CoalescenceSampler;
    use necsim_impls_no_std::cogs::dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler as DispersalSampler;
    use necsim_impls_no_std::cogs::event_sampler::independent::IndependentEventSampler as EventSampler;
    use necsim_impls_no_std::cogs::habitat::in_memory::InMemoryHabitat as Habitat;
    use necsim_impls_no_std::cogs::lineage_reference::in_memory::InMemoryLineageReference as LineageReference;
    use necsim_impls_no_std::cogs::lineage_store::incoherent::in_memory::IncoherentInMemoryLineageStore as LineageStore;
    use necsim_impls_no_std::rng::aes::AesRng as Rng;

    simulate_generic(
        simulation_c_ptr
            as *mut <Simulation<
                Habitat,
                DispersalSampler,
                LineageReference,
                LineageStore<_>,
                CoalescenceSampler<_, _, _>,
                EventSampler<_, _, _, _>,
                ActiveLineageSampler<_, _, _, _>,
            > as RustToCuda>::CudaRepresentation,
        cuda_rng_c_ptr as *mut <CudaRng<Rng> as RustToCuda>::CudaRepresentation,
        max_steps,
    )
}

unsafe fn simulate_generic<
    H: Habitat + RustToCuda,
    D: DispersalSampler<H> + RustToCuda,
    R: LineageReference<H> + DeviceCopy,
    S: IncoherentLineageStore<H, R> + RustToCuda,
    C: CoalescenceSampler<H, R, S> + RustToCuda,
    E: EventSampler<H, D, R, S, C> + RustToCuda,
    A: ActiveLineageSampler<H, D, R, S, C, E> + RustToCuda,
    G: Rng,
>(
    simulation_ptr: *mut <Simulation<H, D, R, S, C, E, A> as RustToCuda>::CudaRepresentation,
    cuda_rng_ptr: *mut <CudaRng<G> as RustToCuda>::CudaRepresentation,
    max_steps: usize,
) {
    Simulation::with_borrow_from_rust_mut(simulation_ptr, |simulation| {
        CudaRng::with_borrow_from_rust_mut(cuda_rng_ptr, |cuda_rng| {
            let mut reporter = NullReporter;

            //println!("{:#?}", simulation);

            let (time, steps) = simulation.simulate_incremental(max_steps, cuda_rng, &mut reporter);

            println!("time = {:?}, steps = {}", F64(time), steps);
        })
    })
}

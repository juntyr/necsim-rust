#![deny(clippy::pedantic)]
#![no_std]
#![feature(const_eval_limit)]
#![const_eval_limit = "1000000000000"]
#![feature(associated_type_bounds)]
#![cfg_attr(target_os = "cuda", feature(abi_ptx))]
#![cfg_attr(target_os = "cuda", feature(alloc_error_handler))]
#![cfg_attr(target_os = "cuda", feature(panic_info_message))]
#![cfg_attr(target_os = "cuda", feature(atomic_from_mut))]
#![cfg_attr(target_os = "cuda", feature(asm_experimental_arch))]
#![cfg_attr(target_os = "cuda", feature(stdsimd))]
#![cfg_attr(target_os = "cuda", feature(control_flow_enum))]

extern crate alloc;

#[cfg(target_os = "cuda")]
use core::ops::ControlFlow;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, Habitat, ImmigrationEntry,
        LineageStore, MathsCore, PrimeableRng, Rng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::{MinSpeciationTrackingEventSampler, SpeciationSample},
};

use rust_cuda::common::RustToCuda;

#[rust_cuda::common::kernel(
    pub use link_kernel! as impl SimulatableKernel<SimulationKernelArgs> for SimulationKernel
)]
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn simulate<
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
>(
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    simulation: &mut ShallowCopy<
        necsim_core::simulation::Simulation<M, H, G, S, X, D, C, T, N, E, I, A>,
    >,
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    task_list: &mut ShallowCopy<
        necsim_impls_cuda::value_buffer::ValueBuffer<necsim_core::lineage::Lineage, true, true>,
    >,
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    event_buffer_reporter: &mut ShallowCopy<
        necsim_impls_cuda::event_buffer::EventBuffer<ReportSpeciation, ReportDispersal>,
    >,
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    min_spec_sample_buffer: &mut ShallowCopy<
        necsim_impls_cuda::value_buffer::ValueBuffer<SpeciationSample, false, true>,
    >,
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    next_event_time_buffer: &mut ShallowCopy<
        necsim_impls_cuda::value_buffer::ValueBuffer<necsim_core_bond::PositiveF64, false, true>,
    >,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_time_max: &core::sync::atomic::AtomicU64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_steps_sum: &core::sync::atomic::AtomicU64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    max_steps: u64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    max_next_event_time: necsim_core_bond::NonNegativeF64,
) {
    task_list.with_value_for_core(|task| {
        // Discard the prior task (the simulation is just a temporary local copy)
        core::mem::drop(
            simulation
                .active_lineage_sampler_mut()
                .replace_active_lineage(task),
        );

        // Discard the prior sample (the simulation is just a temporary local copy)
        simulation.event_sampler_mut().replace_min_speciation(None);

        let mut final_next_event_time = None;

        let (time, steps) = simulation.simulate_incremental_early_stop(
            |_, steps, next_event_time| {
                final_next_event_time = Some(next_event_time);

                if steps >= max_steps || next_event_time >= max_next_event_time {
                    ControlFlow::Break(())
                } else {
                    ControlFlow::Continue(())
                }
            },
            event_buffer_reporter,
        );

        next_event_time_buffer.put_value_for_core(final_next_event_time);

        if steps > 0 {
            total_time_max.fetch_max(time.get().to_bits(), core::sync::atomic::Ordering::Relaxed);
            total_steps_sum.fetch_add(steps, core::sync::atomic::Ordering::Relaxed);
        }

        min_spec_sample_buffer
            .put_value_for_core(simulation.event_sampler_mut().replace_min_speciation(None));

        simulation
            .active_lineage_sampler_mut()
            .replace_active_lineage(None)
    });
}

#[rust_cuda::common::kernel(
    pub use link_sort_kernel! as impl SortableKernel<SortKernelArgs> for SortKernel
)]
pub fn sort_events_step<ReportSpeciation: Boolean, ReportDispersal: Boolean>(
    #[kernel(pass = LendRustToCuda, jit)] event_buffer_reporter: &mut ShallowCopy<
        necsim_impls_cuda::event_buffer::EventBuffer<ReportSpeciation, ReportDispersal>,
    >,
    #[kernel(pass = SafeDeviceCopy)] size: usize,
    #[kernel(pass = SafeDeviceCopy)] stride: usize,
) {
    // Safety: size and stride are the same on every CUDA thread
    unsafe {
        event_buffer_reporter.odd_even_sort_events_step(size, stride);
    }
}

// #[rust_cuda::common::kernel(
//     pub use link_even_odd_sort_kernel! as impl EvenOddSortableKernel<EvenOddSortKernelArgs> for EvenOddSortKernel
// )]
// pub fn even_odd_sort_events_step<ReportSpeciation: Boolean, ReportDispersal: Boolean>(
//     #[kernel(pass = LendRustToCuda, jit)] event_buffer_reporter: &mut ShallowCopy<
//         necsim_impls_cuda::event_buffer::EventBuffer<ReportSpeciation, ReportDispersal>,
//     >,
//     #[kernel(pass = SafeDeviceCopy)] size: usize,
//     #[kernel(pass = SafeDeviceCopy)] stride: usize,
// ) {
//     // Safety: size and stride are the same on every CUDA thread
//     unsafe {
//         event_buffer_reporter.odd_even_sort_events_step(size, stride);
//     }
// }

// #[rust_cuda::common::kernel(
//     pub use link_bitonic_sort_kernel! as impl BitonicSortableKernel<BitonicSortKernelArgs> for BitonicSortKernel
// )]
// pub fn bitonic_sort_events_step<ReportSpeciation: Boolean, ReportDispersal: Boolean>(
//     #[kernel(pass = LendRustToCuda, jit)] event_buffer_reporter: &mut ShallowCopy<
//         necsim_impls_cuda::event_buffer::EventBuffer<ReportSpeciation, ReportDispersal>,
//     >,
//     #[kernel(pass = SafeDeviceCopy)] size: usize,
//     #[kernel(pass = SafeDeviceCopy)] stride: usize,
// ) {
//     // Safety: size and stride are the same on every CUDA thread
//     unsafe {
//         event_buffer_reporter.bitonic_sort_events_step(size, stride);
//     }
// }

#[cfg(target_os = "cuda")]
mod cuda_prelude {
    use core::arch::nvptx;

    use rust_cuda::device::utils;

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
}

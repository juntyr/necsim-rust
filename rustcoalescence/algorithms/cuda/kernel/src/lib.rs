#![deny(clippy::pedantic)]
#![no_std]
#![feature(core_intrinsics)]
#![cfg_attr(target_os = "cuda", feature(abi_ptx))]
#![cfg_attr(target_os = "cuda", feature(alloc_error_handler))]
#![cfg_attr(target_os = "cuda", feature(panic_info_message))]
#![cfg_attr(target_os = "cuda", feature(atomic_from_mut))]
#![cfg_attr(target_os = "cuda", feature(asm))]
#![cfg_attr(target_os = "cuda", feature(stdsimd))]
#![cfg_attr(target_os = "cuda", feature(const_float_bits_conv))]
#![allow(clippy::type_complexity)]

extern crate alloc;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, F64Core, Habitat, ImmigrationEntry,
        LineageReference, LineageStore, PrimeableRng, SpeciationProbability, TurnoverRate,
    },
    reporter::boolean::Boolean,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::singular::SingularActiveLineageSampler,
    event_sampler::tracking::{MinSpeciationTrackingEventSampler, SpeciationSample},
};

use rust_cuda::common::RustToCuda;

#[rust_cuda::common::kernel(pub use link_kernel! as impl Kernel<KernelArgs> for SimulationKernel)]
#[allow(clippy::too_many_arguments)]
pub fn simulate<
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
    A: SingularActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I> + RustToCuda,
    ReportSpeciation: Boolean,
    ReportDispersal: Boolean,
>(
    #[rustfmt::skip]
    #[kernel(pass = LendRustToCuda, jit)]
    simulation: &mut ShallowCopy<
        necsim_core::simulation::Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A>,
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

                steps >= max_steps || next_event_time >= max_next_event_time
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

#[derive(Clone, Debug)]
pub enum NvptxF64Core {}

// TODO: Ensure consistency for cached calculations on the CPU,
//        maybe guard with linker error?
impl F64Core for NvptxF64Core {
    #[inline]
    fn floor(x: f64) -> f64 {
        unsafe { core::intrinsics::floorf64(x) }
    }

    #[inline]
    fn ceil(x: f64) -> f64 {
        unsafe { core::intrinsics::ceilf64(x) }
    }

    #[inline]
    fn ln(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::logf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            const FRAC_1_LOG2_E: f64 = 1.0_f64 / core::f64::consts::LOG2_E;

            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            asm!("lg2.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            // f / log_2(e)
            f64::from(f) * FRAC_1_LOG2_E
        }
    }

    #[inline]
    fn exp(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::expf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = (x * core::f64::consts::LOG2_E) as f32;
            let f: f32;

            asm!("ex2.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn sqrt(x: f64) -> f64 {
        unsafe { core::intrinsics::sqrtf64(x) }
    }

    #[inline]
    fn sin(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::sinf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            asm!("sin.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn cos(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::cosf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            #[allow(clippy::cast_possible_truncation)]
            let x: f32 = x as f32;
            let f: f32;

            asm!("cos.approx.f32 {}, {};", out(reg32) f, in(reg32) x, options(pure, nomem, nostack));

            f64::from(f)
        }
    }

    #[inline]
    fn round(x: f64) -> f64 {
        #[cfg(not(target_os = "cuda"))]
        unsafe {
            core::intrinsics::roundf64(x)
        }
        #[cfg(target_os = "cuda")]
        unsafe {
            const ROUND_TRUNC_OFFSET: f64 = 0.5_f64 - 0.25_f64 * f64::EPSILON;

            let offset: f64;
            asm!("copysign {}, {}, {};", out(reg64) offset, in(reg64) x, const ROUND_TRUNC_OFFSET.to_bits(), options(pure, nomem, nostack));

            let overshot = x + offset;

            let round: f64;
            asm!("cvt.rzi.f64.f64 {}, {};", out(reg64) round, in(reg64) overshot, options(pure, nomem, nostack));

            round
        }
    }
}

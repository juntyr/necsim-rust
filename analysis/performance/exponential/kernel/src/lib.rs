#![deny(clippy::pedantic)]
#![no_std]
#![cfg_attr(target_os = "cuda", feature(abi_ptx))]
#![cfg_attr(target_os = "cuda", feature(alloc_error_handler))]
#![cfg_attr(target_os = "cuda", feature(panic_info_message))]
#![cfg_attr(target_os = "cuda", feature(asm_experimental_arch))]
#![cfg_attr(target_os = "cuda", feature(stdsimd))]
#![allow(clippy::type_complexity)]

extern crate alloc;

use core::sync::atomic::AtomicU64;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, TurnoverRate},
    landscape::Location,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

#[cfg(target_os = "cuda")]
mod benchmark;

#[cfg(target_os = "cuda")]
mod clock;

#[cfg(target_os = "cuda")]
mod sample;

#[rust_cuda::common::kernel(pub use link_poisson_kernel! as impl PoissonKernel<PoissonKernelArgs> for BenchmarkPoissonKernel)]
pub fn benchmark_poisson(
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    seed: u64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    lambda: PositiveF64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    delta_t: PositiveF64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    limit: &[u8; 16],
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_cycles_sum: &AtomicU64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_time_sum: &AtomicU64,
) {
    use necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::poisson::PoissonEventTimeSampler;

    benchmark::inter_event_times(
        PoissonEventTimeSampler::new(delta_t),
        seed,
        lambda,
        u128::from_le_bytes(*limit),
        total_cycles_sum,
        total_time_sum,
    );
}

#[rust_cuda::common::kernel(pub use link_exp_kernel! as impl ExpKernel<ExpKernelArgs> for BenchmarkExpKernel)]
pub fn benchmark_exp(
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    seed: u64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    lambda: PositiveF64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    delta_t: PositiveF64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    limit: &[u8; 16],
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_cycles_sum: &AtomicU64,
    #[rustfmt::skip]
    #[kernel(pass = SafeDeviceCopy)]
    total_time_sum: &AtomicU64,
) {
    use necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::exp::ExpEventTimeSampler;

    benchmark::inter_event_times(
        ExpEventTimeSampler::new(delta_t),
        seed,
        lambda,
        u128::from_le_bytes(*limit),
        total_cycles_sum,
        total_time_sum,
    );
}

#[derive(Debug)]
pub struct UniformTurnoverRate {
    turnover_rate: PositiveF64,
}

impl UniformTurnoverRate {
    #[must_use]
    pub fn new(turnover_rate: PositiveF64) -> Self {
        Self { turnover_rate }
    }
}

#[contracts::contract_trait]
impl Backup for UniformTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            turnover_rate: self.turnover_rate,
        }
    }
}

#[contracts::contract_trait]
impl<M: MathsCore, H: Habitat<M>> TurnoverRate<M, H> for UniformTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> NonNegativeF64 {
        // Use a volatile read to ensure that the turnover rate cannot be
        //  optimised out of this benchmark test

        unsafe { core::ptr::read_volatile(&self.turnover_rate) }.into()
    }
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

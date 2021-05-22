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
extern crate contracts;

use core::sync::atomic::{AtomicU64, Ordering};

use necsim_core::{
    cogs::{Backup, Habitat, PrimeableRng, RngCore, TurnoverRate},
    landscape::{IndexedLocation, Location},
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::event_time_sampler::{
        exp::ExpEventTimeSampler, poisson::PoissonEventTimeSampler, EventTimeSampler,
    },
    habitat::non_spatial::NonSpatialHabitat,
    rng::wyhash::WyHash,
};

use rust_cuda::{
    common::{DeviceBoxConst, DeviceBoxMut},
    device::{nvptx, utils},
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

/// A predefined, read-only 64-bit unsigned cycle counter.
#[inline]
#[must_use]
pub fn clock_counter() -> u64 {
    let counter: u64;
    unsafe { asm!("mov.u64  {}, %clock64;", out(reg64) counter, options(nostack)) };
    counter
}

/// A predefined, 64-bit global nanosecond timer.
#[inline]
#[must_use]
pub fn clock_timer_ns() -> u64 {
    let timer: u64;
    unsafe { asm!("mov.u64  {}, %globaltimer;", out(reg64) timer, options(nostack)) };
    timer
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn benchmark_poisson(
    seed: u64,
    lambda: PositiveF64,
    delta_t: PositiveF64,
    limit: DeviceBoxConst<u128>,
    total_cycles_sum: DeviceBoxMut<u64>,
    total_time_sum: DeviceBoxMut<u64>,
) {
    benchmark_inter_event_times(
        PoissonEventTimeSampler::new(delta_t),
        seed,
        lambda,
        *limit.as_ref(),
        total_cycles_sum,
        total_time_sum,
    )
}

#[no_mangle]
pub unsafe extern "ptx-kernel" fn benchmark_exp(
    seed: u64,
    lambda: PositiveF64,
    delta_t: PositiveF64,
    limit: DeviceBoxConst<u128>,
    total_cycles_sum: DeviceBoxMut<u64>,
    total_time_sum: DeviceBoxMut<u64>,
) {
    benchmark_inter_event_times(
        ExpEventTimeSampler::new(delta_t),
        seed,
        lambda,
        *limit.as_ref(),
        total_cycles_sum,
        total_time_sum,
    )
}

#[inline]
fn benchmark_inter_event_times<
    E: EventTimeSampler<NonSpatialHabitat, WyHash, UniformTurnoverRate>,
>(
    event_time_sampler: E,
    seed: u64,
    lambda: PositiveF64,
    limit: u128,
    mut total_cycles_sum: DeviceBoxMut<u64>,
    mut total_time_sum: DeviceBoxMut<u64>,
) {
    let habitat = NonSpatialHabitat::new((1, 1), 1);
    let rng = WyHash::seed_from_u64(seed + (utils::index() as u64));
    let turnover_rate = UniformTurnoverRate {
        turnover_rate: lambda,
    };
    let indexed_location = IndexedLocation::new(Location::new(0, 0), 0);

    let (cycles, time) = sample_exponential_inter_event_times(
        habitat,
        rng,
        turnover_rate,
        event_time_sampler,
        indexed_location,
        limit,
    );

    AtomicU64::from_mut(total_cycles_sum.as_mut()).fetch_add(cycles, Ordering::Relaxed);
    AtomicU64::from_mut(total_time_sum.as_mut()).fetch_add(time, Ordering::Relaxed);
}

#[inline]
#[allow(clippy::needless_pass_by_value)]
fn sample_exponential_inter_event_times<
    H: Habitat,
    G: PrimeableRng,
    T: TurnoverRate<H>,
    E: EventTimeSampler<H, G, T>,
>(
    habitat: H,
    mut rng: G,
    turnover_rate: T,
    event_time_sampler: E,
    indexed_location: IndexedLocation,
    limit: u128,
) -> (u64, u64) {
    let mut last_event_time = NonNegativeF64::zero();

    let time_start = clock_timer_ns();
    let cycle_start = clock_counter();

    for _ in 0..limit {
        let next_event_time = event_time_sampler.next_event_time_at_indexed_location_weakly_after(
            &indexed_location,
            last_event_time,
            &habitat,
            &mut rng,
            &turnover_rate,
        );

        last_event_time = next_event_time;
    }

    let cycle_finish = clock_counter();
    let time_finish = clock_timer_ns();

    (
        time_finish.wrapping_sub(time_start),
        cycle_finish.wrapping_sub(cycle_start),
    )
}

#[derive(Debug)]
pub struct UniformTurnoverRate {
    turnover_rate: PositiveF64,
}

#[contract_trait]
impl Backup for UniformTurnoverRate {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            turnover_rate: self.turnover_rate,
        }
    }
}

#[contract_trait]
impl<H: Habitat> TurnoverRate<H> for UniformTurnoverRate {
    #[must_use]
    #[inline]
    fn get_turnover_rate_at_location(&self, _location: &Location, _habitat: &H) -> NonNegativeF64 {
        // Use a volatile read to ensure that the turnover rate cannot be
        //  optimised out of this benchmark test

        unsafe { core::ptr::read_volatile(&self.turnover_rate) }.into()
    }
}

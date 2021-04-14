#![deny(clippy::pedantic)]
#![feature(const_maybe_uninit_assume_init)]
#![feature(option_result_unwrap_unchecked)]

use necsim_core::cogs::RngCore;
use necsim_impls_no_std::cogs::rng::fixedseahash::FixedSeaHash;

static mut RNG: Option<FixedSeaHash> = None;

#[no_mangle]
pub extern "C" fn thisrng_seed(seed: u64) {
    unsafe { RNG = Some(FixedSeaHash::seed_from_u64(seed)) };
}

#[no_mangle]
pub extern "C" fn thisrng() -> u64 {
    unsafe { RNG.as_mut().unwrap_unchecked().sample_u64() }
}

#[no_mangle]
#[used]
pub static name: &str = concat!("necsim-rng", "\0");

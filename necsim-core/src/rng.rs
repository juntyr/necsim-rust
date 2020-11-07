use crate::intrinsics::{floor, ln};

use core::convert::AsMut;
use core::default::Default;
use core::ptr::copy_nonoverlapping;

#[allow(clippy::module_name_repetitions)]
pub trait RngCore: Sized + Clone {
    type Seed: AsMut<[u8]> + Default + Sized;

    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self;

    #[must_use]
    fn seed_from_u64(mut state: u64) -> Self {
        // Implementation from:
        // https://docs.rs/rand/0.7.3/rand/trait.SeedableRng.html#method.seed_from_u64

        // We use PCG32 to generate a u32 sequence, and copy to the seed
        const MUL: u64 = 6_364_136_223_846_793_005_u64;
        const INC: u64 = 11_634_580_027_462_260_723_u64;

        let mut seed = Self::Seed::default();
        for chunk in seed.as_mut().chunks_mut(4) {
            // We advance the state first (to get away from the input value,
            // in case it has low Hamming Weight).
            state = state.wrapping_mul(MUL).wrapping_add(INC);

            // Use PCG output function with to_le to generate x:
            #[allow(clippy::cast_possible_truncation)]
            let xorshifted = (((state >> 18) ^ state) >> 27) as u32;
            #[allow(clippy::cast_possible_truncation)]
            let rot = (state >> 59) as u32;
            let x = xorshifted.rotate_right(rot).to_le();

            unsafe {
                let p = &x as *const u32 as *const u8;
                copy_nonoverlapping(p, chunk.as_mut_ptr(), chunk.len());
            }
        }

        Self::from_seed(seed)
    }

    #[must_use]
    fn sample_u64(&mut self) -> u64;
}

pub trait IncoherentRngCore: RngCore {
    type Prime: AsMut<[u8]> + Sized;

    fn prime_with(&mut self, prime: Self::Prime);
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait Rng: RngCore {
    #[must_use]
    #[inline]
    #[debug_ensures(ret >= 0.0_f64 && ret <= 1.0_f64, "samples U(0.0, 1.0)")]
    fn sample_uniform(&mut self) -> f64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        ((self.sample_u64() >> 11) as f64)
            * f64::from_bits(0x3CA0_0000_0000_0000_u64) //0x1.0p-53
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length, "samples U(0, length - 1)")]
    fn sample_index(&mut self, length: usize) -> usize {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = floor(self.sample_uniform() * (length as f64)) as usize;
        index
    }

    #[must_use]
    #[inline]
    #[debug_requires(lambda > 0.0, "lambda > 0.0")]
    #[debug_ensures(ret >= 0.0, "samples Exp(lambda)")]
    fn sample_exponential(&mut self, lambda: f64) -> f64 {
        -ln(self.sample_uniform()) / lambda
    }

    #[must_use]
    #[inline]
    #[debug_requires(
        probability >= 0.0_f64 && probability <= 1.0_f64,
        "0.0 <= probability <= 1.0"
    )]
    fn sample_event(&mut self, probability: f64) -> bool {
        self.sample_uniform() < probability
    }
}

impl<R: RngCore> Rng for R {}

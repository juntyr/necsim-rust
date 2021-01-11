use crate::{
    cogs::Habitat,
    intrinsics::{cos, floor, ln, sin, sqrt},
    landscape::IndexedLocation,
};

use core::{convert::AsMut, default::Default, ptr::copy_nonoverlapping};

#[allow(clippy::module_name_repetitions)]
pub trait RngCore: Sized + Clone + core::fmt::Debug {
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

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait RngSampler: RngCore {
    #[must_use]
    #[inline]
    #[debug_ensures((0.0_f64..=1.0_f64).contains(&ret), "samples U(0.0, 1.0)")]
    fn sample_uniform(&mut self) -> f64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        ((self.sample_u64() >> 11) as f64)
            * f64::from_bits(0x3CA0_0000_0000_0000_u64) // 0x1.0p-53
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
    #[debug_ensures(ret < length, "samples U(0, length - 1)")]
    fn sample_index_u32(&mut self, length: u32) -> u32 {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = floor(self.sample_uniform() * f64::from(length)) as u32;
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
        (0.0_f64..=1.0_f64).contains(&probability),
        "0.0 <= probability <= 1.0"
    )]
    fn sample_event(&mut self, probability: f64) -> bool {
        self.sample_uniform() < probability
    }

    #[must_use]
    #[inline]
    fn sample_2d_standard_normal(&mut self) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 = self.sample_uniform();
        let u1 = self.sample_uniform();

        let r = sqrt(-2.0_f64 * ln(u0));
        let theta = -core::f64::consts::TAU * u1;

        (r * sin(theta), r * cos(theta))
    }

    #[must_use]
    #[inline]
    #[debug_requires(sigma >= 0.0_f64, "standard deviation sigma must be non-negative")]
    fn sample_2d_normal(&mut self, mu: f64, sigma: f64) -> (f64, f64) {
        let (z0, z1) = self.sample_2d_standard_normal();

        (z0 * sigma + mu, z1 * sigma + mu)
    }
}

impl<R: RngCore> RngSampler for R {}

#[allow(clippy::module_name_repetitions)]
pub trait PrimeableRng<H: Habitat>: RngCore {
    fn prime_with_habitat(
        &mut self,
        habitat: &H,
        indexed_location: &IndexedLocation,
        time_index: u64,
    ) {
        self.prime_with(
            habitat.map_indexed_location_to_u64_injective(indexed_location),
            time_index,
        )
    }

    fn prime_with(&mut self, location_index: u64, time_index: u64);
}

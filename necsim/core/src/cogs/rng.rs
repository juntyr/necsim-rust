use core::{
    convert::AsMut,
    default::Default,
    num::{NonZeroU128, NonZeroU32, NonZeroUsize},
    ptr::copy_nonoverlapping,
};

use serde::{de::DeserializeOwned, Serialize};

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveF64};

use crate::{
    cogs::{Habitat, MathsCore},
    landscape::IndexedLocation,
};

#[allow(clippy::module_name_repetitions)]
pub trait RngCore<M: MathsCore>:
    crate::cogs::Backup + Sized + Clone + core::fmt::Debug + Serialize + DeserializeOwned
{
    type Seed: AsMut<[u8]> + Default + Sized;

    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self;

    #[must_use]
    fn sample_u64(&mut self) -> u64;
}

#[allow(clippy::module_name_repetitions)]
pub trait SeedableRng<M: MathsCore>: RngCore<M> {
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
                let p = (&x as *const u32).cast::<u8>();
                copy_nonoverlapping(p, chunk.as_mut_ptr(), chunk.len());
            }
        }

        Self::from_seed(seed)
    }
}

impl<M: MathsCore, R: RngCore<M>> SeedableRng<M> for R {}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait RngSampler<M: MathsCore>: RngCore<M> {
    #[must_use]
    #[inline]
    fn sample_uniform(&mut self) -> ClosedUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        let u01 = ((self.sample_u64() >> 11) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { ClosedUnitF64::new_unchecked(u01) }
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index(&mut self, length: NonZeroUsize) -> usize {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(self.sample_uniform().get() * (length.get() as f64)) as usize;
        index.min(length.get() - 1)
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index_u32(&mut self, length: NonZeroU32) -> u32 {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(self.sample_uniform().get() * f64::from(length.get())) as u32;
        index.min(length.get() - 1)
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index_u128(&mut self, length: NonZeroU128) -> u128 {
        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(self.sample_uniform().get() * (length.get() as f64)) as u128;
        index.min(length.get() - 1)
    }

    #[must_use]
    #[inline]
    fn sample_exponential(&mut self, lambda: PositiveF64) -> NonNegativeF64 {
        let exp = -M::ln(self.sample_uniform().get()) / lambda.get();

        unsafe { NonNegativeF64::new_unchecked(exp) }
    }

    #[must_use]
    #[inline]
    fn sample_event(&mut self, probability: ClosedUnitF64) -> bool {
        self.sample_uniform().get() < probability.get()
    }

    #[must_use]
    #[inline]
    fn sample_2d_standard_normal(&mut self) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 = self.sample_uniform();
        let u1 = self.sample_uniform();

        let r = M::sqrt(-2.0_f64 * M::ln(u0.get()));
        let theta = -core::f64::consts::TAU * u1.get();

        (r * M::sin(theta), r * M::cos(theta))
    }

    #[must_use]
    #[inline]
    fn sample_2d_normal(&mut self, mu: f64, sigma: NonNegativeF64) -> (f64, f64) {
        let (z0, z1) = self.sample_2d_standard_normal();

        (z0 * sigma.get() + mu, z1 * sigma.get() + mu)
    }
}

impl<M: MathsCore, R: RngCore<M>> RngSampler<M> for R {}

#[allow(clippy::module_name_repetitions)]
pub trait PrimeableRng<M: MathsCore>: RngCore<M> {
    fn prime_with(&mut self, location_index: u64, time_index: u64);
}

#[allow(clippy::module_name_repetitions)]
pub trait HabitatPrimeableRng<M: MathsCore, H: Habitat<M>>: PrimeableRng<M> {
    #[inline]
    fn prime_with_habitat(
        &mut self,
        habitat: &H,
        indexed_location: &IndexedLocation,
        time_index: u64,
    ) {
        self.prime_with(
            habitat.map_indexed_location_to_u64_injective(indexed_location),
            time_index,
        );
    }
}

impl<M: MathsCore, R: PrimeableRng<M>, H: Habitat<M>> HabitatPrimeableRng<M, H> for R {}

#[allow(clippy::module_name_repetitions)]
pub trait SplittableRng<M: MathsCore>: RngCore<M> {
    fn split(self) -> (Self, Self);

    fn split_to_stream(self, stream: u64) -> Self;
}

use core::{
    convert::AsMut,
    default::Default,
    num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize},
    ptr::copy_nonoverlapping,
};

use serde::{de::DeserializeOwned, Serialize};

use necsim_core_bond::{
    ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64, PositiveF64,
};

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
                let p = core::ptr::addr_of!(x).cast::<u8>();
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
    /// Samples a uniform sample within `[0.0, 1.0)`, i.e. `0.0 <= X < 1.0`
    fn sample_uniform_closed_open(&mut self) -> ClosedOpenUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        let u01 = ((self.sample_u64() >> 11) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { ClosedOpenUnitF64::new_unchecked(u01) }
    }

    #[must_use]
    #[inline]
    /// Samples a uniform sample within `(0.0, 1.0]`, i.e. `0.0 < X <= 1.0`
    fn sample_uniform_open_closed(&mut self) -> OpenClosedUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        let u01 =
            (((self.sample_u64() >> 11) + 1) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { OpenClosedUnitF64::new_unchecked(u01) }
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index(&mut self, length: NonZeroUsize) -> usize {
        #[cfg(target_pointer_width = "32")]
        #[allow(clippy::cast_possible_truncation)]
        {
            self.sample_index_u32(unsafe { NonZeroU32::new_unchecked(length.get() as u32) })
                as usize
        }
        #[cfg(target_pointer_width = "64")]
        #[allow(clippy::cast_possible_truncation)]
        {
            self.sample_index_u64(unsafe { NonZeroU64::new_unchecked(length.get() as u64) })
                as usize
        }
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index_u32(&mut self, length: NonZeroU32) -> u32 {
        // TODO: Check if delegation to `sample_index_u64` is faster

        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        const LOWER_MASK: u64 = !0 >> 32;

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw = self.sample_u64();

            let sample_check_lo = (raw & LOWER_MASK) * u64::from(length.get());

            #[allow(clippy::cast_possible_truncation)]
            if (sample_check_lo as u32) <= acceptance_zone {
                return (sample_check_lo >> 32) as u32;
            }

            let sample_check_hi = (raw >> 32) * u64::from(length.get());

            #[allow(clippy::cast_possible_truncation)]
            if (sample_check_hi as u32) <= acceptance_zone {
                return (sample_check_hi >> 32) as u32;
            }
        }
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index_u64(&mut self, length: NonZeroU64) -> u64 {
        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw = self.sample_u64();

            let sample_check = u128::from(raw) * u128::from(length.get());

            #[allow(clippy::cast_possible_truncation)]
            if (sample_check as u64) <= acceptance_zone {
                return (sample_check >> 64) as u64;
            }
        }
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    fn sample_index_u128(&mut self, length: NonZeroU128) -> u128 {
        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        const LOWER_MASK: u128 = !0 >> 64;

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw_hi = u128::from(self.sample_u64());
            let raw_lo = u128::from(self.sample_u64());

            // 256-bit multiplication (hi, lo) = (raw_hi, raw_lo) * length
            let mut low = raw_lo * (length.get() & LOWER_MASK);
            let mut t = low >> 64;
            low &= LOWER_MASK;
            t += raw_hi * (length.get() & LOWER_MASK);
            low += (t & LOWER_MASK) << 64;
            let mut high = t >> 64;
            t = low >> 64;
            low &= LOWER_MASK;
            t += (length.get() >> 64) * raw_lo;
            low += (t & LOWER_MASK) << 64;
            high += t >> 64;
            high += raw_hi * (length.get() >> 64);

            let sample = high;
            let check = low;

            if check <= acceptance_zone {
                return sample;
            }
        }
    }

    #[must_use]
    #[inline]
    fn sample_exponential(&mut self, lambda: PositiveF64) -> NonNegativeF64 {
        // Inverse transform sample: X = -ln(U(0,1]) / lambda
        -self.sample_uniform_open_closed().ln::<M>() / lambda
    }

    #[must_use]
    #[inline]
    fn sample_event(&mut self, probability: ClosedUnitF64) -> bool {
        // if probability == 1, then U[0, 1) always < 1.0
        // if probability == 0, then U[0, 1) never < 0.0
        self.sample_uniform_closed_open() < probability
    }

    #[must_use]
    #[inline]
    fn sample_2d_standard_normal(&mut self) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 = self.sample_uniform_open_closed();
        let u1 = self.sample_uniform_closed_open();

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
    #[must_use]
    fn split(self) -> (Self, Self);

    #[must_use]
    fn split_to_stream(self, stream: u64) -> Self;
}

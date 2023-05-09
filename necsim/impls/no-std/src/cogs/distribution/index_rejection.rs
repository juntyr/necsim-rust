use core::num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize};

use necsim_core::cogs::{
    distribution::{IndexU128, IndexU32, IndexU64, IndexUsize, Length, RawDistribution},
    DistributionSampler, MathsCore, RngCore,
};

#[allow(clippy::module_name_repetitions)]
pub struct IndexRejectionSampler;

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexUsize>
    for IndexRejectionSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    #[inline]
    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Length(length): Length<NonZeroUsize>,
    ) -> usize {
        #[allow(clippy::cast_possible_truncation)]
        if cfg!(target_pointer_width = "32") {
            IndexU32::sample_raw_with::<M, R, Self>(
                rng,
                self,
                Length(unsafe { NonZeroU32::new_unchecked(length.get() as u32) }),
            ) as usize
        } else {
            IndexU64::sample_raw_with::<M, R, Self>(
                rng,
                self,
                Length(unsafe { NonZeroU64::new_unchecked(length.get() as u64) }),
            ) as usize
        }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU32> for IndexRejectionSampler {
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Length(length): Length<NonZeroU32>,
    ) -> u32 {
        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        const LOWER_MASK: u64 = !0 >> 32;

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw = rng.sample_u64();

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
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU64> for IndexRejectionSampler {
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Length(length): Length<NonZeroU64>,
    ) -> u64 {
        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw = rng.sample_u64();

            let sample_check = u128::from(raw) * u128::from(length.get());

            #[allow(clippy::cast_possible_truncation)]
            if (sample_check as u64) <= acceptance_zone {
                return (sample_check >> 64) as u64;
            }
        }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU128>
    for IndexRejectionSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Length(length): Length<NonZeroU128>,
    ) -> u128 {
        // Adapted from:
        // https://docs.rs/rand/0.8.4/rand/distributions/uniform/trait.UniformSampler.html#method.sample_single

        const LOWER_MASK: u128 = !0 >> 64;

        // Conservative approximation of the acceptance zone
        let acceptance_zone = (length.get() << length.leading_zeros()).wrapping_sub(1);

        loop {
            let raw_hi = u128::from(rng.sample_u64());
            let raw_lo = u128::from(rng.sample_u64());

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
}

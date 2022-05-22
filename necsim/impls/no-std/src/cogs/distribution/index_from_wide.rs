use core::num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize};

use necsim_core::cogs::{
    distribution::{IndexU128, IndexU32, IndexU64, IndexUsize, Length, RawDistribution},
    DistributionSampler, MathsCore, RngCore,
};

#[allow(clippy::module_name_repetitions)]
pub struct IndexFromWideMulSampler;

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexUsize>
    for IndexFromWideMulSampler
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

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU32>
    for IndexFromWideMulSampler
{
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
        // Sample U(0, length - 1) using a widening multiplication
        // Note: Some slight bias is traded for only needing one u64 sample
        // Note: Should optimise to a single 64 bit (high-only) multiplication
        #[allow(clippy::cast_possible_truncation)]
        {
            (((u128::from(rng.sample_u64()) * u128::from(length.get())) >> 64) & u128::from(!0_u32))
                as u32
        }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU64>
    for IndexFromWideMulSampler
{
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
        // Sample U(0, length - 1) using a widening multiplication
        // Note: Some slight bias is traded for only needing one u64 sample
        // Note: Should optimise to a single 64 bit (high-only) multiplication
        #[allow(clippy::cast_possible_truncation)]
        {
            (((u128::from(rng.sample_u64()) * u128::from(length.get())) >> 64) & u128::from(!0_u64))
                as u64
        }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, IndexU128>
    for IndexFromWideMulSampler
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
        // Sample U(0, length - 1) using a widening multiplication
        // Note: Some slight bias is traded for only needing one u128 sample

        const LOWER_MASK: u128 = !0 >> 64;

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
        // low-only: low &= LOWER_MASK;
        t += (length.get() >> 64) * raw_lo;
        // low-only: low += (t & LOWER_MASK) << 64;
        high += t >> 64;
        high += raw_hi * (length.get() >> 64);

        high
    }
}

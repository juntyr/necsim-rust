use core::num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize};

use necsim_core::cogs::{
    distribution::{
        IndexU128, IndexU32, IndexU64, IndexUsize, Length, RawDistribution, UniformClosedOpenUnit,
    },
    DistributionSampler, MathsCore, RngCore,
};

#[allow(clippy::module_name_repetitions)]
pub struct IndexFromUnitSampler;

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexUsize> for IndexFromUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Length(length): Length<NonZeroUsize>,
    ) -> usize {
        let u01 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        // Safety: U[0, 1) * length in [0, 2^[32/64]) is a valid [u32/u64]
        //         since (1 - 2^-53) * 2^[32/64] <= (2^[32/64] - 1)
        #[allow(clippy::cast_precision_loss)]
        let index =
            unsafe { M::floor(u01.get() * (length.get() as f64)).to_int_unchecked::<usize>() };

        if cfg!(target_pointer_width = "32") {
            // Note: [0, 2^32) is losslessly represented in f64
            index
        } else {
            // Note: Ensure index < length despite
            //       usize->f64->usize precision loss
            index.min(length.get() - 1)
        }
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU32> for IndexFromUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Length(length): Length<NonZeroU32>,
    ) -> u32 {
        let u01 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        // Safety: U[0, 1) * length in [0, 2^32) is losslessly represented
        //         in both f64 and u32
        unsafe { M::floor(u01.get() * f64::from(length.get())).to_int_unchecked::<u32>() }
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU64> for IndexFromUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Length(length): Length<NonZeroU64>,
    ) -> u64 {
        let u01 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        // Safety: U[0, 1) * length in [0, 2^64) is a valid u64
        //         since (1 - 2^-53) * 2^64 <= (2^64 - 1)
        #[allow(clippy::cast_precision_loss)]
        let index =
            unsafe { M::floor(u01.get() * (length.get() as f64)).to_int_unchecked::<u64>() };

        // Note: Ensure index < length despite u64->f64->u64 precision loss
        index.min(length.get() - 1)
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU128> for IndexFromUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Length(length): Length<NonZeroU128>,
    ) -> u128 {
        let u01 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        // Safety: U[0, 1) * length in [0, 2^128) is a valid u128
        //         since (1 - 2^-53) * 2^128 <= (2^128 - 1)
        #[allow(clippy::cast_precision_loss)]
        let index =
            unsafe { M::floor(u01.get() * (length.get() as f64)).to_int_unchecked::<u128>() };

        // Note: Ensure index < length despite u128->f64->u128 precision loss
        index.min(length.get() - 1)
    }
}

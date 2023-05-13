use core::num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize};

use necsim_core::cogs::{
    distribution::{
        Bernoulli, Exponential, IndexU128, IndexU32, IndexU64, IndexUsize, Lambda, Length, Normal,
        Normal2D, Poisson, StandardNormal2D, UniformClosedOpenUnit, UniformOpenClosedUnit,
    },
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64};

use rand_core::RngCore as RandRngCore;
use rand_distr::{
    uniform::{UniformInt as RandUniformInt, UniformSampler as RandUniformSampler},
    Bernoulli as RandBernoulli, Distribution as RandDistribution, Exp1 as RandExp1,
    OpenClosed01 as RandOpenClosed01, Poisson as RandPoisson, Standard as RandStandard,
    StandardNormal as RandStandardNormal,
};

#[allow(clippy::module_name_repetitions)]
pub struct RandDistributionSamplers;

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, UniformClosedOpenUnit>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> ClosedOpenUnitF64 {
        let u01: f64 = RandStandard.sample(rng);

        // Safety: Standard samples from [0, 1)
        unsafe { ClosedOpenUnitF64::new_unchecked(u01) }
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, UniformOpenClosedUnit>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> OpenClosedUnitF64 {
        let u01: f64 = RandOpenClosed01.sample(rng);

        // Safety: OpenClosed01 samples from (0, 1]
        unsafe { OpenClosedUnitF64::new_unchecked(u01) }
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, IndexUsize>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Length(length): Length<NonZeroUsize>,
    ) -> usize {
        RandUniformInt::<usize>::sample_single(0, length.get(), rng)
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, IndexU32>
    for RandDistributionSamplers
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
        RandUniformInt::<u32>::sample_single(0, length.get(), rng)
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, IndexU64>
    for RandDistributionSamplers
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
        RandUniformInt::<u64>::sample_single(0, length.get(), rng)
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, IndexU128>
    for RandDistributionSamplers
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
        RandUniformInt::<u128>::sample_single(0, length.get(), rng)
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, Exponential>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        _samplers: &S,
        Lambda(lambda): Lambda,
    ) -> NonNegativeF64 {
        let exp1: f64 = RandExp1.sample(rng);

        // Safety: Exp1 samples from [0, +inf)
        (unsafe { NonNegativeF64::new_unchecked(exp1) }) / lambda
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, Poisson>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, Lambda(lambda): Lambda) -> u64 {
        // Safety: PositiveF64 asserts that lambda > 0
        let poisson = unsafe { RandPoisson::new(lambda.get()).unwrap_unchecked() };

        // Note: rust clamps f64 as u64 to [0, 2^64 - 1]
        #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
        {
            poisson.sample(rng) as u64
        }
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, Bernoulli>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, probability: ClosedUnitF64) -> bool {
        // Safety: ClosedUnitF64 asserts that probability is in [0.0, 1.0]
        let bernoulli = unsafe { RandBernoulli::new(probability.get()).unwrap_unchecked() };

        bernoulli.sample(rng)
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, StandardNormal2D>
    for RandDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> (f64, f64) {
        (
            RandStandardNormal.sample(rng),
            RandStandardNormal.sample(rng),
        )
    }
}

impl<M: MathsCore, R: RngCore + RandRngCore, S> DistributionSampler<M, R, S, Normal2D>
    for RandDistributionSamplers
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
        Normal { mu, sigma }: Normal,
    ) -> (f64, f64) {
        let (z0, z1): (f64, f64) = (
            RandStandardNormal.sample(rng),
            RandStandardNormal.sample(rng),
        );

        (z0 * sigma.get() + mu, z1 * sigma.get() + mu)
    }
}

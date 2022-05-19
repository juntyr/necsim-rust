use core::num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize};

use necsim_core_bond::{
    ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64, PositiveF64,
};

use crate::cogs::{MathsCore, RngCore, Samples};

pub trait Distribution {
    type Parameters;
    type Sample;
}

#[allow(clippy::module_name_repetitions)]
pub trait SampledDistribution: Distribution {
    fn sample_with<M: MathsCore, R: Samples<M, Self>>(
        rng: &mut R,
        params: Self::Parameters,
    ) -> Self::Sample;

    fn sample<M: MathsCore, R: Samples<M, Self>>(rng: &mut R) -> Self::Sample
    where
        Self: Distribution<Parameters = ()>,
    {
        Self::sample_with(rng, ())
    }
}

impl<D: Distribution> SampledDistribution for D {
    fn sample_with<M: MathsCore, R: Samples<M, Self>>(
        rng: &mut R,
        params: Self::Parameters,
    ) -> Self::Sample {
        rng.sample_with(params)
    }
}

#[allow(clippy::module_name_repetitions)]
pub trait DistributionSampler<M: MathsCore, R: RngCore, S, D: Distribution> {
    type ConcreteSampler: DistributionSampler<M, R, S, D>;

    #[must_use]
    fn concrete(&self) -> &Self::ConcreteSampler;

    #[must_use]
    fn sample_with(&self, rng: &mut R, samplers: &S, params: D::Parameters) -> D::Sample;

    #[must_use]
    fn sample(&self, rng: &mut R, samplers: &S) -> D::Sample
    where
        D: Distribution<Parameters = ()>,
    {
        self.sample_with(rng, samplers, ())
    }
}

pub enum UniformClosedOpenUnit {}

impl Distribution for UniformClosedOpenUnit {
    type Parameters = ();
    type Sample = ClosedOpenUnitF64;
}

pub enum UniformOpenClosedUnit {}

impl Distribution for UniformOpenClosedUnit {
    type Parameters = ();
    type Sample = OpenClosedUnitF64;
}

pub enum IndexUsize {}

pub struct Length<T>(pub T);

impl Distribution for IndexUsize {
    type Parameters = Length<NonZeroUsize>;
    type Sample = usize;
}

pub enum IndexU32 {}

impl Distribution for IndexU32 {
    type Parameters = Length<NonZeroU32>;
    type Sample = u32;
}

pub enum IndexU64 {}

impl Distribution for IndexU64 {
    type Parameters = Length<NonZeroU64>;
    type Sample = u64;
}

pub enum IndexU128 {}

impl Distribution for IndexU128 {
    type Parameters = Length<NonZeroU128>;
    type Sample = u128;
}

pub struct Lambda(pub PositiveF64);

pub enum Exponential {}

impl Distribution for Exponential {
    type Parameters = Lambda;
    type Sample = NonNegativeF64;
}

pub enum Poisson {}

impl Distribution for Poisson {
    type Parameters = Lambda;
    type Sample = usize;
}

pub enum Bernoulli {}

impl Distribution for Bernoulli {
    type Parameters = ClosedUnitF64;
    type Sample = bool;
}

pub enum StandardNormal2D {}

impl Distribution for StandardNormal2D {
    type Parameters = ();
    type Sample = (f64, f64);
}

pub struct Normal {
    pub mu: f64,
    pub sigma: NonNegativeF64,
}

pub enum Normal2D {}

impl Distribution for Normal2D {
    type Parameters = Normal;
    type Sample = (f64, f64);
}

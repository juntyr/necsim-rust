use core::{
    marker::PhantomData,
    num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize},
};

use necsim_core::cogs::{
    distribution::{
        Bernoulli, Exponential, IndexU128, IndexU32, IndexU64, IndexUsize, Lambda, Length, Normal,
        Normal2D, Poisson, StandardNormal2D, UniformClosedOpenUnit, UniformOpenClosedUnit,
    },
    Backup, DistributionSampler, MathsCore, Rng, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64};

use crate::cogs::distribution::{
    bernoulli_64b::Bernoulli64BitSampler, exp_inversion::ExponentialInverseTransformSampler,
    index_from_unit::IndexFromUnitSampler, normal2d::Normal2dSampler,
    poisson_inversion::PoissonInverseTransformOrNormalSampler,
    std_normal2d_box_muller::StandardNormal2DBoxMullerSampler,
    uniform_53b_unit::Uniform53BitUnitSampler,
};

#[derive(Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
#[layout(free = "M")]
#[repr(transparent)]
pub struct SimpleRng<M: MathsCore, R: RngCore> {
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: RngCore> From<R> for SimpleRng<M, R> {
    fn from(inner: R) -> Self {
        Self {
            inner,
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: RngCore> SimpleRng<M, R> {
    pub fn into_inner(self) -> R {
        self.inner
    }
}

#[contract_trait]
impl<M: MathsCore, R: RngCore> Backup for SimpleRng<M, R> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: RngCore> Rng<M> for SimpleRng<M, R> {
    type Generator = R;
    type Sampler = SimpleDistributionSamplers<M, R>;

    fn generator(&mut self) -> &mut Self::Generator {
        &mut self.inner
    }

    fn map_generator<F: FnOnce(Self::Generator) -> Self::Generator>(self, map: F) -> Self {
        let SimpleRng { inner, marker } = self;

        SimpleRng {
            inner: map(inner),
            marker,
        }
    }

    fn with<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q {
        let samplers = SimpleDistributionSamplers {
            u01: Uniform53BitUnitSampler,
            index: IndexFromUnitSampler,
            exp: ExponentialInverseTransformSampler,
            poisson: PoissonInverseTransformOrNormalSampler,
            bernoulli: Bernoulli64BitSampler,
            std_normal_2d: StandardNormal2DBoxMullerSampler,
            normal_2d: Normal2dSampler,
            _marker: PhantomData::<(M, R)>,
        };

        inner(&mut self.inner, &samplers)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct SimpleDistributionSamplers<M: MathsCore, R: RngCore> {
    u01: Uniform53BitUnitSampler,
    index: IndexFromUnitSampler,
    exp: ExponentialInverseTransformSampler,
    poisson: PoissonInverseTransformOrNormalSampler,
    bernoulli: Bernoulli64BitSampler,
    std_normal_2d: StandardNormal2DBoxMullerSampler,
    normal_2d: Normal2dSampler,
    _marker: PhantomData<(M, R)>,
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformClosedOpenUnit>
    for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = Uniform53BitUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.u01
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: ()) -> ClosedOpenUnitF64 {
        DistributionSampler::<M, R, _, UniformClosedOpenUnit>::sample_distribution(
            &self.u01, rng, samplers, params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformOpenClosedUnit>
    for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = Uniform53BitUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.u01
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: ()) -> OpenClosedUnitF64 {
        DistributionSampler::<M, R, _, UniformOpenClosedUnit>::sample_distribution(
            &self.u01, rng, samplers, params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexUsize> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        params: Length<NonZeroUsize>,
    ) -> usize {
        DistributionSampler::<M, R, _, IndexUsize>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU32> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU32>) -> u32 {
        DistributionSampler::<M, R, _, IndexU32>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU64> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU64>) -> u64 {
        DistributionSampler::<M, R, _, IndexU64>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU128> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU128>) -> u128 {
        DistributionSampler::<M, R, _, IndexU128>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformOpenClosedUnit>>
    DistributionSampler<M, R, S, Exponential> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = ExponentialInverseTransformSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.exp
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Lambda) -> NonNegativeF64 {
        DistributionSampler::<M, R, _, Exponential>::sample_distribution(
            &self.exp, rng, samplers, params,
        )
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, Normal2D>,
    > DistributionSampler<M, R, S, Poisson> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = PoissonInverseTransformOrNormalSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.poisson
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Lambda) -> u64 {
        DistributionSampler::<M, R, _, Poisson>::sample_distribution(
            &self.poisson,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, Bernoulli>
    for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = Bernoulli64BitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.bernoulli
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: ClosedUnitF64) -> bool {
        DistributionSampler::<M, R, _, Bernoulli>::sample_distribution(
            &self.bernoulli,
            rng,
            samplers,
            params,
        )
    }
}

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, UniformOpenClosedUnit>,
    > DistributionSampler<M, R, S, StandardNormal2D> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = StandardNormal2DBoxMullerSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.std_normal_2d
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: ()) -> (f64, f64) {
        DistributionSampler::<M, R, _, StandardNormal2D>::sample_distribution(
            &self.std_normal_2d,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, StandardNormal2D>>
    DistributionSampler<M, R, S, Normal2D> for SimpleDistributionSamplers<M, R>
{
    type ConcreteSampler = Normal2dSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.normal_2d
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Normal) -> (f64, f64) {
        DistributionSampler::<M, R, _, Normal2D>::sample_distribution(
            &self.normal_2d,
            rng,
            samplers,
            params,
        )
    }
}

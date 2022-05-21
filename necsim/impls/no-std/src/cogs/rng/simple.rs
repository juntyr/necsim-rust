use core::{
    marker::PhantomData,
    num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize},
};

use necsim_core::cogs::{
    distribution::{
        Bernoulli, Exponential, IndexU128, IndexU32, IndexU64, IndexUsize, Lambda, Length, Normal,
        Normal2D, Poisson, RawDistribution, StandardNormal2D, UniformClosedOpenUnit,
        UniformOpenClosedUnit,
    },
    Backup, DistributionSampler, MathsCore, Rng, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64};

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
    type Sampler = SimplerDistributionSamplers<M, R>;

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
        let samplers = SimplerDistributionSamplers {
            _marker: PhantomData::<(M, R)>,
        };

        inner(&mut self.inner, &samplers)
    }
}

#[allow(clippy::module_name_repetitions)]
pub struct SimplerDistributionSamplers<M: MathsCore, R: RngCore> {
    _marker: PhantomData<(M, R)>,
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformClosedOpenUnit>
    for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> ClosedOpenUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        let u01 = ((rng.sample_u64() >> 11) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { ClosedOpenUnitF64::new_unchecked(u01) }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformOpenClosedUnit>
    for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> OpenClosedUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        let u01 =
            (((rng.sample_u64() >> 11) + 1) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { OpenClosedUnitF64::new_unchecked(u01) }
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexUsize> for SimplerDistributionSamplers<M, R>
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

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(u01.get() * (length.get() as f64)) as usize;

        // Safety in case of f64 rounding errors
        index.min(length.get() - 1)
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU32> for SimplerDistributionSamplers<M, R>
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

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let index = M::floor(u01.get() * f64::from(length.get())) as u32;

        // Safety in case of f64 rounding errors
        index.min(length.get() - 1)
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU64> for SimplerDistributionSamplers<M, R>
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

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(u01.get() * (length.get() as f64)) as u64;

        // Safety in case of f64 rounding errors
        index.min(length.get() - 1)
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU128> for SimplerDistributionSamplers<M, R>
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

        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = M::floor(u01.get() * (length.get() as f64)) as u128;

        // Safety in case of f64 rounding errors
        index.min(length.get() - 1)
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformOpenClosedUnit>>
    DistributionSampler<M, R, S, Exponential> for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Lambda(lambda): Lambda,
    ) -> NonNegativeF64 {
        let u01 = UniformOpenClosedUnit::sample_raw(rng, samplers);

        // Inverse transform sample: X = -ln(U(0,1]) / lambda
        -u01.ln::<M>() / lambda
    }
}

impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, Normal2D>,
    > DistributionSampler<M, R, S, Poisson> for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, Lambda(lambda): Lambda) -> u64 {
        let no_event_probability = M::exp(-lambda.get());

        if no_event_probability <= 0.0_f64 {
            // Fallback in case no_event_probability_per_step underflows
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let normal_as_poisson = Normal2D::sample_raw_with(
                rng,
                samplers,
                Normal {
                    mu: lambda.get(),
                    sigma: NonNegativeF64::from(lambda).sqrt::<M>(),
                },
            )
            .0
            .max(0.0_f64) as u64;

            return normal_as_poisson;
        }

        // https://en.wikipedia.org/wiki/Poisson_distribution#cite_ref-Devroye1986_54-0
        let mut poisson = 0_u64;
        let mut prod = no_event_probability;
        let mut acc = no_event_probability;

        let u = UniformClosedOpenUnit::sample_raw(rng, samplers);

        #[allow(clippy::cast_precision_loss)]
        while u > acc && prod > 0.0_f64 {
            poisson += 1;
            prod *= lambda.get() / (poisson as f64);
            acc += prod;
        }

        poisson
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, Bernoulli> for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, probability: ClosedUnitF64) -> bool {
        let u01 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        // if probability == 1, then U[0, 1) always < 1.0
        // if probability == 0, then U[0, 1) never < 0.0
        u01 < probability
    }
}

impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, UniformOpenClosedUnit>,
    > DistributionSampler<M, R, S, StandardNormal2D> for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, _params: ()) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 = UniformOpenClosedUnit::sample_raw(rng, samplers);
        let u1 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        let r = M::sqrt(-2.0_f64 * M::ln(u0.get()));
        let theta = -core::f64::consts::TAU * u1.get();

        (r * M::sin(theta), r * M::cos(theta))
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, StandardNormal2D>>
    DistributionSampler<M, R, S, Normal2D> for SimplerDistributionSamplers<M, R>
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Normal { mu, sigma }: Normal,
    ) -> (f64, f64) {
        let (z0, z1) = StandardNormal2D::sample_raw(rng, samplers);

        (z0 * sigma.get() + mu, z1 * sigma.get() + mu)
    }
}

use core::{
    marker::PhantomData,
    num::{NonZeroU128, NonZeroU32, NonZeroU64, NonZeroUsize},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use necsim_core::cogs::{
    distribution::{
        Bernoulli, Exponential, IndexU128, IndexU32, IndexU64, IndexUsize, Lambda, Length, Normal,
        Normal2D, Poisson, StandardNormal2D, UniformClosedOpenUnit, UniformOpenClosedUnit,
    },
    Backup, DistributionSampler, MathsCore, Rng, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64};

#[derive(Clone, Debug, TypeLayout)]
#[allow(clippy::module_name_repetitions)]
#[layout(free = "M")]
#[repr(transparent)]
pub struct SimpleRng<M: MathsCore, R: RngCore> {
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: RngCore> Serialize for SimpleRng<M, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, M: MathsCore, R: RngCore> Deserialize<'de> for SimpleRng<M, R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = R::deserialize(deserializer)?;

        Ok(Self {
            inner,
            marker: PhantomData::<M>,
        })
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

impl<M: MathsCore, R: RngCore> RngCore for SimpleRng<M, R> {
    type Seed = R::Seed;

    #[must_use]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: R::from_seed(seed),
            marker: PhantomData::<M>,
        }
    }

    #[must_use]
    fn sample_u64(&mut self) -> u64 {
        self.inner.sample_u64()
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

    fn with_rng<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q {
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

    fn sample_with(&self, rng: &mut R, _samplers: &S, _params: ()) -> ClosedOpenUnitF64 {
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

    fn sample_with(&self, rng: &mut R, _samplers: &S, _params: ()) -> OpenClosedUnitF64 {
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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Length<NonZeroUsize>) -> usize {
        let length = params.0;

        let u01: ClosedOpenUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU32>) -> u32 {
        let length = params.0;

        let u01: ClosedOpenUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU64>) -> u64 {
        let length = params.0;

        let u01: ClosedOpenUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU128>) -> u128 {
        let length = params.0;

        let u01: ClosedOpenUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Lambda) -> NonNegativeF64 {
        let lambda = params.0;

        let u01: OpenClosedUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Lambda) -> u64 {
        let lambda = params.0;
        let no_event_probability = M::exp(-lambda.get());

        if no_event_probability <= 0.0_f64 {
            // Fallback in case no_event_probability_per_step underflows
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let normal_as_poisson = DistributionSampler::<M, R, S, Normal2D>::sample_with(
                samplers,
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

        let u =
            DistributionSampler::<M, R, S, UniformClosedOpenUnit>::sample(samplers, rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: ClosedUnitF64) -> bool {
        let probability = params;

        let u01: ClosedOpenUnitF64 = samplers.sample(rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, _params: ()) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 =
            DistributionSampler::<M, R, S, UniformOpenClosedUnit>::sample(samplers, rng, samplers);
        let u1 =
            DistributionSampler::<M, R, S, UniformClosedOpenUnit>::sample(samplers, rng, samplers);

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

    fn sample_with(&self, rng: &mut R, samplers: &S, params: Normal) -> (f64, f64) {
        let (z0, z1) = samplers.sample(rng, samplers);

        (
            z0 * params.sigma.get() + params.mu,
            z1 * params.sigma.get() + params.mu,
        )
    }
}

/*#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
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

impl<M: MathsCore, R: RngCore<M>> RngSampler<M> for R {}*/

use core::marker::PhantomData;

use necsim_core::cogs::{Distribution, DistributionSampler, MathsCore, Rng, RngCore};

use const_type_layout::TypeGraphLayout;
use rust_cuda::safety::StackOnly;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, rust_cuda::common::LendRustToCuda)]
#[cuda(free = "M", free = "R")]
pub struct CudaRng<M: MathsCore, R>
where
    R: Rng<M> + StackOnly + ~const TypeGraphLayout,
{
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Clone for CudaRng<M, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> From<R> for CudaRng<M, R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self {
            inner: rng,
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> RngCore
    for CudaRng<M, R>
{
    type Seed = <R as RngCore>::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: R::from_seed(seed),
            marker: PhantomData::<M>,
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.inner.sample_u64()
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Rng<M> for CudaRng<M, R> {
    type Generator = R::Generator;
    type Sampler = R::Sampler;

    fn generator(&mut self) -> &mut Self::Generator {
        self.inner.generator()
    }

    fn sample_with<D: Distribution>(&mut self, params: D::Parameters) -> D::Sample
    where
        Self::Sampler: DistributionSampler<M, Self::Generator, Self::Sampler, D>,
    {
        self.inner.sample_with(params)
    }
}

impl<M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Serialize for CudaRng<M, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, M: MathsCore, R: Rng<M> + StackOnly + ~const TypeGraphLayout> Deserialize<'de> for CudaRng<M, R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = R::deserialize(deserializer)?;

        Ok(Self {
            inner,
            marker: PhantomData::<M>,
        })
    }
}

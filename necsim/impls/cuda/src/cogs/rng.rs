use core::marker::PhantomData;

use necsim_core::cogs::{Backup, MathsCore, PrimeableRng, RngCore};

use const_type_layout::TypeLayout;
use rust_cuda::safety::StackOnly;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, rust_cuda::common::LendRustToCuda)]
pub struct CudaRng<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> {
    inner: R,
    marker: PhantomData<M>,
}

impl<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> Clone for CudaRng<M, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> From<R> for CudaRng<M, R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self {
            inner: rng,
            marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> Backup for CudaRng<M, R> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> RngCore<M> for CudaRng<M, R> {
    type Seed = <R as RngCore<M>>::Seed;

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

impl<M: MathsCore, R: PrimeableRng<M> + StackOnly + TypeLayout> PrimeableRng<M> for CudaRng<M, R> {
    #[inline]
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.inner.prime_with(location_index, time_index);
    }
}

impl<M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> Serialize for CudaRng<M, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, M: MathsCore, R: RngCore<M> + StackOnly + TypeLayout> Deserialize<'de> for CudaRng<M, R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = R::deserialize(deserializer)?;

        Ok(Self {
            inner,
            marker: PhantomData::<M>,
        })
    }
}

use core::marker::PhantomData;

use necsim_core::cogs::{Backup, F64Core, PrimeableRng, RngCore};

use rust_cuda::memory::StackOnly;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, rust_cuda::common::LendRustToCuda)]
pub struct CudaRng<F: F64Core, R: RngCore<F> + StackOnly> {
    inner: R,
    marker: PhantomData<F>,
}

impl<F: F64Core, R: RngCore<F> + StackOnly> Clone for CudaRng<F, R> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
            marker: PhantomData,
        }
    }
}

impl<F: F64Core, R: RngCore<F> + StackOnly> From<R> for CudaRng<F, R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self {
            inner: rng,
            marker: PhantomData,
        }
    }
}

#[contract_trait]
impl<F: F64Core, R: RngCore<F> + StackOnly> Backup for CudaRng<F, R> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<F: F64Core, R: RngCore<F> + StackOnly> RngCore<F> for CudaRng<F, R> {
    type Seed = <R as RngCore<F>>::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self {
            inner: R::from_seed(seed),
            marker: PhantomData,
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.inner.sample_u64()
    }
}

impl<F: F64Core, R: PrimeableRng<F> + StackOnly> PrimeableRng<F> for CudaRng<F, R> {
    #[inline]
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.inner.prime_with(location_index, time_index);
    }
}

impl<F: F64Core, R: RngCore<F> + StackOnly> Serialize for CudaRng<F, R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.inner.serialize(serializer)
    }
}

impl<'de, F: F64Core, R: RngCore<F> + StackOnly> Deserialize<'de> for CudaRng<F, R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = R::deserialize(deserializer)?;

        Ok(Self {
            inner,
            marker: PhantomData,
        })
    }
}

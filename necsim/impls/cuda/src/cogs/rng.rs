use necsim_core::cogs::{Backup, PrimeableRng, RngCore};
use rust_cuda::utils::stack::StackOnly;

use serde::{Deserialize, Deserializer, Serialize, Serializer};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, rust_cuda::common::RustToCudaAsRust)]
pub struct CudaRng<R: RngCore + StackOnly>(R);

impl<R: RngCore + StackOnly> From<R> for CudaRng<R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self(rng)
    }
}

#[contract_trait]
impl<R: RngCore + StackOnly> Backup for CudaRng<R> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<R: RngCore + StackOnly> RngCore for CudaRng<R> {
    type Seed = <R as RngCore>::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(R::from_seed(seed))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.0.sample_u64()
    }
}

impl<R: PrimeableRng + StackOnly> PrimeableRng for CudaRng<R> {
    #[inline]
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.0.prime_with(location_index, time_index);
    }
}

impl<R: RngCore + StackOnly> Serialize for CudaRng<R> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize(serializer)
    }
}

impl<'de, R: RngCore + StackOnly> Deserialize<'de> for CudaRng<R> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let inner = R::deserialize(deserializer)?;

        Ok(Self(inner))
    }
}

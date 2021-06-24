use necsim_core::cogs::{Backup, PrimeableRng, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, rust_cuda::common::RustToCuda, rust_cuda::host::LendToCuda)]
pub struct CudaRng<R: RngCore>(R);

impl<R: RngCore> From<R> for CudaRng<R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self(rng)
    }
}

#[contract_trait]
impl<R: RngCore> Backup for CudaRng<R> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<R: RngCore> RngCore for CudaRng<R> {
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

impl<R: PrimeableRng> PrimeableRng for CudaRng<R> {
    #[inline]
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.0.prime_with(location_index, time_index);
    }
}

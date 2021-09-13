use necsim_core::cogs::{Backup, PrimeableRng, RngCore};
use rust_cuda::utils::stack::StackOnly;

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
    type State = <R as RngCore>::State;

    #[must_use]
    fn from_state(state: Self::State) -> Self {
        Self(R::from_state(state))
    }

    #[must_use]
    fn into_state(self) -> Self::State {
        R::into_state(self.0)
    }

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

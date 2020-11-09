use necsim_core::cogs::{PrimeableRng, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, RustToCuda, LendToCuda)]
pub struct CudaRng<R: RngCore>(R);

impl<R: RngCore> From<R> for CudaRng<R> {
    #[must_use]
    #[inline]
    fn from(rng: R) -> Self {
        Self(rng)
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
    type Prime = <R as PrimeableRng>::Prime;

    fn prime_with(&mut self, prime: Self::Prime) {
        self.0.prime_with(prime)
    }
}

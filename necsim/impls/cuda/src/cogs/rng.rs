use necsim_core::cogs::{Habitat, PrimeableRng, RngCore};

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

impl<H: Habitat, R: PrimeableRng<H>> PrimeableRng<H> for CudaRng<R> {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.0.prime_with(location_index, time_index)
    }
}

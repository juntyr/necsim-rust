use necsim_core::rng::RngCore;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, RustToCuda, LendToCuda)]
pub struct CudaRng<R: RngCore>(R);

impl<R: RngCore> CudaRng<R> {
    #[must_use]
    #[inline]
    pub fn from_cloned(rng: &mut R) -> Self {
        Self(rng.clone())
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

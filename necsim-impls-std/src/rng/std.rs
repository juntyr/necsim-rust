use rand::rngs::StdRng as StdRngImpl;
use rand::RngCore;
use rand::SeedableRng;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct StdRng(StdRngImpl);

impl necsim_core::rng::RngCore for StdRng {
    type Seed = <StdRngImpl as SeedableRng>::Seed;

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(StdRngImpl::from_seed(seed))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.0.next_u64()
    }
}

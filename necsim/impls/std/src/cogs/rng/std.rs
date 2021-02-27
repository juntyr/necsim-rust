use rand::{rngs::StdRng as StdRngImpl, RngCore, SeedableRng};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct StdRng(StdRngImpl);

#[contract_trait]
impl necsim_core::cogs::Backup for StdRng {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl necsim_core::cogs::RngCore for StdRng {
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

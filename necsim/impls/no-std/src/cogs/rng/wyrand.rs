use nanorand::{WyRand as WyImpl, RNG};

use necsim_core::cogs::{Backup, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct WyRand(WyImpl);

#[contract_trait]
impl Backup for WyRand {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for WyRand {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(WyImpl::new_seed(u64::from_le_bytes(seed)))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.0.generate()
    }
}

impl core::fmt::Debug for WyRand {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("WyRand").finish_non_exhaustive()
    }
}

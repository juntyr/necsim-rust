use pcg::Pcg as PcgImpl;

use necsim_core::cogs::{Backup, RngCore, SplittableRng};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct Pcg(PcgImpl);

#[contract_trait]
impl Backup for Pcg {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for Pcg {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(PcgImpl::new(u64::from_le_bytes(seed), 0))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        rand_core::RngCore::next_u64(&mut self.0)
    }
}

impl SplittableRng for Pcg {
    fn split(mut self) -> (Self, Self) {
        let seed = self.sample_u64();

        let left = Self(PcgImpl::new(seed, 0));
        let right = Self(PcgImpl::new(seed, 1));

        (left, right)
    }

    fn split_to_stream(mut self, stream: u64) -> Self {
        Self(PcgImpl::new(self.sample_u64(), stream))
    }
}

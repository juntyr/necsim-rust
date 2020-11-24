use nanorand::{WyRand, RNG};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone)]
pub struct WyRng(WyRand);

impl necsim_core::cogs::RngCore for WyRng {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        Self(WyRand::new_seed(u64::from_le_bytes(seed)))
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.0.generate()
    }
}

impl core::fmt::Debug for WyRng {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_tuple("WyRng").field(&"WyRand").finish()
    }
}

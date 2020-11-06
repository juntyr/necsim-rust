use nanorand::WyRand;
use nanorand::RNG;

pub struct WyRng(WyRand);

#[contract_trait]
impl necsim_core::rng::Core for WyRng {
    fn sample_uniform(&mut self) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        (self.0.generate_range(0_u64, u64::MAX) as f64)
            / #[allow(clippy::cast_precision_loss)]
            (u64::MAX as f64)
    }
}

impl WyRng {
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(WyRand::new_seed(seed))
    }
}

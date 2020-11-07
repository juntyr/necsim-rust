use nanorand::WyRand;
use nanorand::RNG;

pub struct WyRng(WyRand);

#[contract_trait]
impl necsim_core::rng::Core for WyRng {
    fn sample_uniform(&mut self) -> f64 {
        // U(0.0, 1.0) generation from:
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        #[allow(clippy::cast_precision_loss)]
        ((self.0.generate::<u64>() >> 11) as f64)
            * f64::from_bits(0x3CA0_0000_0000_0000_u64) //0x1.0p-53
    }
}

impl WyRng {
    #[must_use]
    pub fn from_seed(seed: u64) -> Self {
        Self(WyRand::new_seed(seed))
    }
}

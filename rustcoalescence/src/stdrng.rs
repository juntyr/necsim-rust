use rand::rngs::StdRng;
use rand::Rng;
use rand::SeedableRng;

pub struct NewStdRng(StdRng);

impl necsim_core::rng::Core for NewStdRng {
    fn sample_uniform(&mut self) -> f64 {
        self.0.gen_range(0.0_f64, 1.0_f64)
    }
}

impl NewStdRng {
    pub fn from_seed(seed: u64) -> Self {
        Self(StdRng::seed_from_u64(seed))
    }
}

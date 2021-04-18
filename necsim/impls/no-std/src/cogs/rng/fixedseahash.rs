use necsim_core::cogs::{Backup, PrimeableRng, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct FixedSeaHash {
    seed: u64,
    location_index: u64,
    time_index: u64,
    state: u64,
}

#[contract_trait]
impl Backup for FixedSeaHash {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for FixedSeaHash {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        let seed = u64::from_le_bytes(seed);

        Self {
            seed,
            location_index: 0_u64,
            time_index: 0_u64,
            state: seed,
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        self.state =
            diffuse(diffuse(self.state ^ self.location_index) ^ self.time_index) ^ self.seed;

        self.state
    }
}

impl PrimeableRng for FixedSeaHash {
    #[inline]
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.location_index = location_index;
        self.time_index = time_index;

        self.state = 0_u64;
    }
}

// https://docs.rs/seahash/4.0.1/src/seahash/helper.rs.html#72-89
#[inline]
const fn diffuse(mut x: u64) -> u64 {
    // These are derived from the PCG RNG's round. Thanks to @Veedrac for proposing
    // this. The basic idea is that we use dynamic shifts, which are determined
    // by the input itself. The shift is chosen by the higher bits, which means
    // that changing those flips the lower bits, which scatters upwards because
    // of the multiplication.

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    let a = x >> 32;
    let b = x >> 60;

    x ^= a >> b;

    x = x.wrapping_mul(0x6eed_0e9d_a4d9_4a4f);

    x
}

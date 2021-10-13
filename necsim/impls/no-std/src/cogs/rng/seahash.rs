use core::marker::PhantomData;

use necsim_core::cogs::{Backup, F64Core, PrimeableRng, RngCore};

use serde::{Deserialize, Serialize};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SeaHash<F: F64Core> {
    seed: u64,
    location: u64,
    time: u64,
    offset: u64,
    marker: PhantomData<F>,
}

#[contract_trait]
impl<F: F64Core> Backup for SeaHash<F> {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl<F: F64Core> RngCore<F> for SeaHash<F> {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        let seed = u64::from_le_bytes(seed);

        Self {
            seed,
            location: 0_u64,
            time: 0_u64,
            offset: 0_u64,
            marker: PhantomData::<F>,
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        let sample =
            seahash_diffuse(seahash_diffuse(self.time ^ self.offset) ^ self.location ^ self.seed);

        self.offset += 1;

        sample
    }
}

impl<F: F64Core> PrimeableRng<F> for SeaHash<F> {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.location = location_index;
        self.time = time_index;

        self.offset = 0_u64;
    }
}

#[inline]
const fn seahash_diffuse(mut x: u64) -> u64 {
    // SeaHash diffusion function
    // https://docs.rs/seahash/4.1.0/src/seahash/helper.rs.html#75-92

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

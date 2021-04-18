use necsim_core::cogs::{Backup, PrimeableRng, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct WyHash {
    seed: u64,
    state: u64,
}

#[contract_trait]
impl Backup for WyHash {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for WyHash {
    type Seed = [u8; 8];

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        let seed = u64::from_le_bytes(seed);

        Self { seed, state: seed }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        // Added SeaHash diffuse for better avalanching
        diffuse(wyhash::wyrng(&mut self.state))
    }
}

impl PrimeableRng for WyHash {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        let location_bytes = location_index.to_le_bytes();
        let time_index_bytes = time_index.to_le_bytes();

        // TODO: Check if the byte ordering affects the quality of the RNG
        //       priming -> in order would be closer to CPRNG
        // wyhash swaps 64bit lower and upper half, i.e. to get wyhash to
        // process bytes in order, we would need to supply the indices as
        // 4, 5, 6, 7, 0, 1, 2, 3.
        self.state = wyhash::wyhash(
            &[
                location_bytes[0],
                location_bytes[1],
                location_bytes[2],
                location_bytes[3],
                location_bytes[4],
                location_bytes[5],
                location_bytes[6],
                location_bytes[7],
                time_index_bytes[0],
                time_index_bytes[1],
                time_index_bytes[2],
                time_index_bytes[3],
                time_index_bytes[4],
                time_index_bytes[5],
                time_index_bytes[6],
                time_index_bytes[7],
            ],
            self.seed,
        );
    }
}

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

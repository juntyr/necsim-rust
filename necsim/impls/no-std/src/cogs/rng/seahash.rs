use necsim_core::cogs::{Backup, PrimeableRng, RngCore};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct SeaHash {
    seed: u64,
    location: u64,
    time: u64,
    offset: u64,
}

#[contract_trait]
impl Backup for SeaHash {
    unsafe fn backup_unchecked(&self) -> Self {
        self.clone()
    }
}

impl RngCore for SeaHash {
    type Seed = [u8; 8];
    type State = [u8; 32];

    #[must_use]
    fn from_state(state: Self::State) -> Self {
        let mut seed = <[u8; 8]>::default();
        let mut location = <[u8; 8]>::default();
        let mut time = <[u8; 8]>::default();
        let mut offset = <[u8; 8]>::default();

        seed.copy_from_slice(&state[0..8]);
        location.copy_from_slice(&state[8..16]);
        time.copy_from_slice(&state[16..24]);
        offset.copy_from_slice(&state[24..32]);

        Self {
            seed: u64::from_le_bytes(seed),
            location: u64::from_le_bytes(location),
            time: u64::from_le_bytes(time),
            offset: u64::from_le_bytes(offset),
        }
    }

    #[must_use]
    fn into_state(self) -> Self::State {
        let mut state = [0_u8; 32];

        state[0..8].copy_from_slice(&self.seed.to_le_bytes());
        state[8..16].copy_from_slice(&self.location.to_le_bytes());
        state[16..24].copy_from_slice(&self.time.to_le_bytes());
        state[24..32].copy_from_slice(&self.offset.to_le_bytes());

        state
    }

    #[must_use]
    #[inline]
    fn from_seed(seed: Self::Seed) -> Self {
        let seed = u64::from_le_bytes(seed);

        Self {
            seed,
            location: 0_u64,
            time: 0_u64,
            offset: 0_u64,
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

impl PrimeableRng for SeaHash {
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

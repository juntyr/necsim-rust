use necsim_core::cogs::HabitatToU64Injection;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct XxHash {
    seed: u64,
    state: u64,
}

impl necsim_core::cogs::RngCore for XxHash {
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
        self.state = xxhash_rust::xxh64::xxh64(&self.state.to_le_bytes(), self.seed);
        self.state
    }
}

impl<H: HabitatToU64Injection> necsim_core::cogs::PrimeableRng<H> for XxHash {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        let location_bytes = location_index.to_le_bytes();

        let time_index_bytes = time_index.to_le_bytes();

        self.state = xxhash_rust::xxh64::xxh64(
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

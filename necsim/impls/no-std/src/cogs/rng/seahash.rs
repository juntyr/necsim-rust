use necsim_core::cogs::HabitatToU64Injection;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct SeaHash {
    seed: u64,
    location: u64,
    time: u64,
    offset: u64,
}

impl necsim_core::cogs::RngCore for SeaHash {
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
        }
    }

    #[must_use]
    #[inline]
    fn sample_u64(&mut self) -> u64 {
        let offset_bytes = self.offset.to_le_bytes();
        self.offset += 1;
        seahash::hash_seeded(&offset_bytes, self.time, self.location, self.seed, 0_u64)
    }
}

impl<H: HabitatToU64Injection> necsim_core::cogs::PrimeableRng<H> for SeaHash {
    fn prime_with(&mut self, location_index: u64, time_index: u64) {
        self.location = location_index;
        self.time = time_index;

        self.offset = 0_u64;
    }
}

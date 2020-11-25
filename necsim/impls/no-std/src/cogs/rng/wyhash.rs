use necsim_core::{cogs::HabitatToU64Injection, landscape::IndexedLocation};

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
pub struct WyHash {
    seed: u64,
    state: u64,
}

impl necsim_core::cogs::RngCore for WyHash {
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
        wyhash::wyrng(&mut self.state)
    }
}

impl<H: HabitatToU64Injection> necsim_core::cogs::PrimeableRng<H> for WyHash {
    fn prime_with(&mut self, habitat: &H, indexed_location: &IndexedLocation, time_index: u64) {
        let location_bytes = habitat
            .map_indexed_location_to_u64_injective(indexed_location)
            .to_le_bytes();
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

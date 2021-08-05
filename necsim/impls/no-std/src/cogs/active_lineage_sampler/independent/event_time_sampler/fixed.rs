use necsim_core::{
    cogs::{Habitat, HabitatPrimeableRng, PrimeableRng, TurnoverRate},
    intrinsics::floor,
    landscape::IndexedLocation,
};
use necsim_core_bond::NonNegativeF64;

use super::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
pub struct FixedEventTimeSampler([u8; 0]);

impl Default for FixedEventTimeSampler {
    fn default() -> Self {
        Self([])
    }
}

#[contract_trait]
impl<H: Habitat, G: PrimeableRng, T: TurnoverRate<H>> EventTimeSampler<H, G, T>
    for FixedEventTimeSampler
{
    #[inline]
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        time: NonNegativeF64,
        habitat: &H,
        rng: &mut G,
        turnover_rate: &T,
    ) -> NonNegativeF64 {
        let lambda =
            turnover_rate.get_turnover_rate_at_location(indexed_location.location(), habitat);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let time_step = floor(time.get() * lambda.get()) as u64 + 1;

        rng.prime_with_habitat(habitat, indexed_location, time_step);

        NonNegativeF64::from(time_step) / lambda
    }
}

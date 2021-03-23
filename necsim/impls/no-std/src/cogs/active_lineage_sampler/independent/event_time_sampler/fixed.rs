use necsim_core::{
    cogs::{Habitat, PrimeableRng, TurnoverRate},
    intrinsics::floor,
    landscape::IndexedLocation,
};

use super::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct FixedEventTimeSampler(());

impl Default for FixedEventTimeSampler {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl<H: Habitat, G: PrimeableRng<H>, T: TurnoverRate<H>> EventTimeSampler<H, G, T>
    for FixedEventTimeSampler
{
    #[inline]
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
        rng: &mut G,
        turnover_rate: &T,
    ) -> f64 {
        let lambda =
            turnover_rate.get_turnover_rate_at_location(indexed_location.location(), habitat);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let time_step = floor(time * lambda) as u64 + 1;

        rng.prime_with_habitat(habitat, indexed_location, time_step);

        #[allow(clippy::cast_precision_loss)]
        (time_step as f64)
            / lambda
    }
}

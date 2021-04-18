use necsim_core::{
    cogs::{Habitat, HabitatPrimeableRng, PrimeableRng, RngSampler, TurnoverRate},
    intrinsics::floor,
    landscape::IndexedLocation,
};

use super::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct ExpEventTimeSampler {
    delta_t: f64,
}

impl ExpEventTimeSampler {
    #[debug_requires(delta_t > 0.0_f64, "delta_t is positive")]
    pub fn new(delta_t: f64) -> Self {
        Self { delta_t }
    }
}

#[contract_trait]
impl<H: Habitat, G: PrimeableRng, T: TurnoverRate<H>> EventTimeSampler<H, G, T>
    for ExpEventTimeSampler
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

        let mut event_time: f64 = floor(time / self.delta_t) * self.delta_t;
        let mut time_slice_end: f64 = event_time + self.delta_t;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        rng.prime_with_habitat(
            habitat,
            indexed_location,
            floor(event_time / self.delta_t) as u64,
        );

        let mut sub_index: u64 = u64::MAX;

        loop {
            event_time += rng.sample_exponential(lambda);

            sub_index >>= 1;

            // The time slice is exclusive at time_slice_end
            if event_time >= time_slice_end {
                event_time = time_slice_end;
                time_slice_end = event_time + self.delta_t;
                sub_index = u64::MAX;

                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                rng.prime_with_habitat(
                    habitat,
                    indexed_location,
                    floor(event_time / self.delta_t) as u64,
                );
            } else if event_time > time {
                break;
            }
        }

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        rng.prime_with_habitat(
            habitat,
            indexed_location,
            (floor(event_time / self.delta_t) as u64) + sub_index,
        );

        event_time
    }
}

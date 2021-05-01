use necsim_core::{
    cogs::{Habitat, HabitatPrimeableRng, PrimeableRng, RngSampler, TurnoverRate},
    intrinsics::floor,
    landscape::IndexedLocation,
};

use super::EventTimeSampler;

// 2^64 / PHI
const INV_PHI: u64 = 0x9e37_79b9_7f4a_7c15_u64;

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

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let mut time_step = floor(time / self.delta_t) as u64;

        #[allow(clippy::cast_precision_loss)]
        let mut event_time: f64 = (time_step as f64) * self.delta_t;
        #[allow(clippy::cast_precision_loss)]
        let mut time_slice_end: f64 = ((time_step + 1) as f64) * self.delta_t;

        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        rng.prime_with_habitat(habitat, indexed_location, time_step);

        let mut sub_index: u64 = 0;

        loop {
            event_time += rng.sample_exponential(lambda);

            sub_index += INV_PHI;

            // The time slice is exclusive at time_slice_end
            if event_time >= time_slice_end {
                time_step += 1;
                sub_index = 0;

                event_time = time_slice_end;
                #[allow(clippy::cast_precision_loss)]
                let next_time_slice_end = ((time_step + 1) as f64) * self.delta_t;
                time_slice_end = next_time_slice_end;

                rng.prime_with_habitat(habitat, indexed_location, time_step);
            } else if event_time > time {
                break;
            }
        }

        rng.prime_with_habitat(habitat, indexed_location, time_step + sub_index);

        event_time
    }
}

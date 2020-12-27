use float_next_after::NextAfter;

use necsim_core::{
    cogs::{HabitatToU64Injection, PrimeableRng, RngSampler},
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
    const LAMBDA: f64 = 0.5_f64;

    #[debug_requires(delta_t > 0.0_f64, "delta_t is positive")]
    pub fn new(delta_t: f64) -> Self {
        Self { delta_t }
    }
}

#[contract_trait]
impl<H: HabitatToU64Injection, G: PrimeableRng<H>> EventTimeSampler<H, G> for ExpEventTimeSampler {
    #[inline]
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
        rng: &mut G,
    ) -> f64 {
        let mut event_time: f64 = floor(time / self.delta_t) * self.delta_t;
        let mut time_slice_end: f64 = event_time + self.delta_t;

        loop {
            rng.prime_with_habitat(habitat, indexed_location, event_time.to_bits());

            // next_after is required on every iteration to ensure progress
            event_time =
                (event_time + rng.sample_exponential(Self::LAMBDA)).next_after(f64::INFINITY);

            // The time slice is exclusive at time_slice_end
            if event_time >= time_slice_end {
                event_time = time_slice_end;
                time_slice_end = event_time + self.delta_t;
            } else if event_time > time {
                break;
            }
        }

        rng.prime_with_habitat(habitat, indexed_location, event_time.to_bits());

        event_time
    }
}

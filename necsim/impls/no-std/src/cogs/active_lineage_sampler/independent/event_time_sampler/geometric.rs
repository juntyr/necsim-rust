use necsim_core::{
    cogs::{HabitatToU64Injection, PrimeableRng, RngSampler},
    intrinsics::{exp, floor},
    landscape::IndexedLocation,
};

use super::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct GeometricEventTimeSampler {
    delta_t: f64,
    event_probability_per_step: f64,
}

impl GeometricEventTimeSampler {
    #[debug_requires(delta_t > 0.0_f64, "delta_t is positive")]
    pub fn new(delta_t: f64) -> Self {
        let lambda = 0.5_f64;

        Self {
            delta_t,
            event_probability_per_step: 1.0_f64 - exp(-lambda * delta_t),
        }
    }
}

#[contract_trait]
impl<H: HabitatToU64Injection, G: PrimeableRng<H>> EventTimeSampler<H, G>
    for GeometricEventTimeSampler
{
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
        rng: &mut G,
    ) -> f64 {
        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = floor(time / self.delta_t) as u64 + 1;

        loop {
            rng.prime_with_habitat(habitat, indexed_location, time_step);

            if rng.sample_event(self.event_probability_per_step) {
                break;
            }

            time_step += 1;
        }

        #[allow(clippy::cast_precision_loss)]
        (time_step as f64)
            * self.delta_t
    }
}

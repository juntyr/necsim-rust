use float_next_after::NextAfter;

use necsim_core::{
    cogs::{Habitat, PrimeableRng},
    landscape::IndexedLocation,
};

pub mod exp;
pub mod fixed;
pub mod geometric;
pub mod poisson;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventTimeSampler<H: Habitat, G: PrimeableRng<H>>: Clone + core::fmt::Debug {
    #[debug_requires(time >= 0.0_f64, "event times must be non-negative")]
    #[debug_ensures(ret > time, "the next event will happen after time")]
    #[inline]
    fn next_event_time_at_indexed_location_after(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
        rng: &mut G,
    ) -> f64 {
        let next_event_time = self.next_event_time_at_indexed_location_weakly_after(
            indexed_location,
            time,
            habitat,
            rng,
        );

        let unique_next_event_time: f64 = if next_event_time > time {
            next_event_time
        } else {
            time.next_after(f64::INFINITY)
        };

        unique_next_event_time
    }

    #[debug_requires(time >= 0.0_f64, "event times must be non-negative")]
    #[debug_ensures(ret >= time, "the next event will happen weakly after time")]
    fn next_event_time_at_indexed_location_weakly_after(
        &self,
        indexed_location: &IndexedLocation,
        time: f64,
        habitat: &H,
        rng: &mut G,
    ) -> f64;
}

use necsim_core::{
    cogs::{Habitat, HabitatPrimeableRng, PrimeableRng, RngSampler, TurnoverRate},
    intrinsics::{exp, floor},
    landscape::IndexedLocation,
};

use super::EventTimeSampler;

// 2^64 / PHI
const INV_PHI: u64 = 0x9e37_79b9_7f4a_7c15_u64;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
pub struct PoissonEventTimeSampler {
    delta_t: f64,
}

impl PoissonEventTimeSampler {
    #[debug_requires(delta_t > 0.0_f64, "delta_t is positive")]
    pub fn new(delta_t: f64) -> Self {
        Self { delta_t }
    }
}

#[contract_trait]
impl<H: Habitat, G: PrimeableRng, T: TurnoverRate<H>> EventTimeSampler<H, G, T>
    for PoissonEventTimeSampler
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
        let lambda_per_step = lambda * self.delta_t;
        let no_event_probability_per_step = exp(-lambda_per_step);

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = floor(time / self.delta_t) as u64;

        let (event_time, event_index) = loop {
            rng.prime_with_habitat(habitat, indexed_location, time_step);

            // https://en.wikipedia.org/wiki/Poisson_distribution#cite_ref-Devroye1986_54-0
            let mut x = 0_u32;
            let mut p = no_event_probability_per_step;
            let mut s = p;

            let u = rng.sample_uniform();

            while u > s {
                x += 1;
                p *= lambda_per_step / f64::from(x);
                s += p;
            }

            let number_events_at_time_steps = x;

            let mut next_event = None;

            for event_index in 0..number_events_at_time_steps {
                #[allow(clippy::cast_precision_loss)]
                let event_time = ((time_step as f64) + rng.sample_uniform()) * self.delta_t;

                if event_time > time {
                    next_event = match next_event {
                        Some((later_event_time, _)) if later_event_time > event_time => {
                            Some((event_time, event_index))
                        },
                        Some(next_event) => Some(next_event),
                        None => Some((event_time, event_index)),
                    };
                }
            }

            match next_event {
                Some(next_event) => break next_event,
                None => time_step += 1,
            }
        };

        rng.prime_with_habitat(
            habitat,
            indexed_location,
            time_step + INV_PHI.wrapping_mul(u64::from(event_index + 1)),
        );

        event_time
    }
}

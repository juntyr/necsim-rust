use necsim_core::{
    cogs::{
        distribution::{Lambda, Poisson, UniformClosedOpenUnit},
        rng::HabitatPrimeableRng,
        Distribution, Habitat, MathsCore, PrimeableRng, Rng, Samples, TurnoverRate,
    },
    landscape::IndexedLocation,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

use super::EventTimeSampler;

// 2^64 / PHI
const INV_PHI: u64 = 0x9e37_79b9_7f4a_7c15_u64;

#[allow(clippy::module_name_repetitions)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
pub struct PoissonEventTimeSampler {
    delta_t: PositiveF64,
}

impl PoissonEventTimeSampler {
    #[must_use]
    pub fn new(delta_t: PositiveF64) -> Self {
        Self { delta_t }
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: Rng<M, Generator: PrimeableRng> + Samples<M, Poisson> + Samples<M, UniformClosedOpenUnit>,
        T: TurnoverRate<M, H>,
    > EventTimeSampler<M, H, G, T> for PoissonEventTimeSampler
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
        // Safety: lambda is already >= 0, cannot be 0 if an event occurs at this
        // location
        let lambda_per_step = unsafe { PositiveF64::new_unchecked(lambda.get()) } * self.delta_t;

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = M::floor(time.get() / self.delta_t.get()) as u64;

        let (event_time, event_index) = loop {
            rng.generator()
                .prime_with_habitat(habitat, indexed_location, time_step);

            let number_events_at_time_steps = Poisson::sample_with(rng, Lambda(lambda_per_step));

            let mut next_event = None;

            for event_index in 0..number_events_at_time_steps {
                #[allow(clippy::cast_precision_loss)]
                let event_time = (NonNegativeF64::from(time_step)
                    + NonNegativeF64::from(UniformClosedOpenUnit::sample(rng)))
                    * self.delta_t;

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

        rng.generator().prime_with_habitat(
            habitat,
            indexed_location,
            time_step + INV_PHI.wrapping_mul(event_index + 1),
        );

        event_time
    }
}

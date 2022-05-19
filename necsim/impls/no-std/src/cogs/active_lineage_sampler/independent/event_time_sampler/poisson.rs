use necsim_core::{
    cogs::{
        distribution::{Normal, Normal2D, UniformClosedOpenUnit},
        rng::HabitatPrimeableRng,
        DistributionSampler, Habitat, MathsCore, PrimeableRng, Rng, SampledDistribution,
        TurnoverRate,
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
impl<M: MathsCore, H: Habitat<M>, G: Rng<M, Generator: PrimeableRng>, T: TurnoverRate<M, H>>
    EventTimeSampler<M, H, G, T> for PoissonEventTimeSampler
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>
        + DistributionSampler<M, G::Generator, G::Sampler, Normal2D>,
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
        let lambda_per_step = lambda * self.delta_t;
        let no_event_probability_per_step = M::exp(-lambda_per_step.get());

        #[allow(clippy::cast_possible_truncation)]
        #[allow(clippy::cast_sign_loss)]
        let mut time_step = M::floor(time.get() / self.delta_t.get()) as u64;

        let (event_time, event_index) = loop {
            rng.generator()
                .prime_with_habitat(habitat, indexed_location, time_step);

            let number_events_at_time_steps = if no_event_probability_per_step > 0.0_f64 {
                // https://en.wikipedia.org/wiki/Poisson_distribution#cite_ref-Devroye1986_54-0
                let mut poisson = 0_u32;
                let mut prod = no_event_probability_per_step;
                let mut acc = no_event_probability_per_step;

                let u = UniformClosedOpenUnit::sample(rng);

                while u > acc && prod > 0.0_f64 {
                    poisson += 1;
                    prod *= lambda_per_step.get() / f64::from(poisson);
                    acc += prod;
                }

                poisson
            } else {
                // Fallback in case no_event_probability_per_step underflows
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let normal_as_poisson = Normal2D::sample_with(
                    rng,
                    Normal {
                        mu: lambda_per_step.get(),
                        sigma: lambda_per_step.sqrt::<M>(),
                    },
                )
                .0
                .max(0.0_f64) as u32;

                normal_as_poisson
            };

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
            time_step + INV_PHI.wrapping_mul(u64::from(event_index + 1)),
        );

        event_time
    }
}

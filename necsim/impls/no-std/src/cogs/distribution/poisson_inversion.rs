use necsim_core::cogs::{
    distribution::{Lambda, Normal, Normal2D, Poisson, RawDistribution, UniformClosedOpenUnit},
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::NonNegativeF64;

#[allow(clippy::module_name_repetitions)]
pub struct PoissonInverseTransformOrNormalSampler;

impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, Normal2D>,
    > DistributionSampler<M, R, S, Poisson> for PoissonInverseTransformOrNormalSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, Lambda(lambda): Lambda) -> u64 {
        let no_event_probability = M::exp(-lambda.get());

        if no_event_probability <= 0.0_f64 {
            // Fallback in case no_event_probability_per_step underflows
            // Note: rust clamps f64 as u64 to [0, 2^64 - 1]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
            let normal_as_poisson = Normal2D::sample_raw_with(
                rng,
                samplers,
                Normal {
                    mu: lambda.get(),
                    sigma: NonNegativeF64::from(lambda).sqrt::<M>(),
                },
            )
            .0 as u64;

            return normal_as_poisson;
        }

        // https://en.wikipedia.org/w/index.php?title=Poisson_distribution&oldid=1088559556#cite_ref-Devroye1986_61-0
        let mut poisson = 0_u64;
        let mut prod = no_event_probability;
        let mut acc = no_event_probability;

        let u = UniformClosedOpenUnit::sample_raw(rng, samplers);

        #[allow(clippy::cast_precision_loss)]
        while u > acc && prod > 0.0_f64 {
            poisson += 1;
            prod *= lambda.get() / (poisson as f64);
            acc += prod;
        }

        poisson
    }
}

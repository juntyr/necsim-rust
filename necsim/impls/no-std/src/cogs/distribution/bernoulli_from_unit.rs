use necsim_core::cogs::{
    distribution::{Bernoulli, RawDistribution, UniformClosedOpenUnit},
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::ClosedUnitF64;

#[allow(clippy::module_name_repetitions)]
pub struct BernoulliFromUnitSampler;

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, Bernoulli> for BernoulliFromUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, probability: ClosedUnitF64) -> bool {
        // if probability == 1, then U[0, 1) always < 1.0
        // if probability == 0, then U[0, 1) never < 0.0
        UniformClosedOpenUnit::sample_raw(rng, samplers) < probability
    }
}

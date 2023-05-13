use necsim_core::cogs::{
    distribution::{Exponential, Lambda, RawDistribution, UniformOpenClosedUnit},
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::NonNegativeF64;

#[allow(clippy::module_name_repetitions)]
pub struct ExponentialInverseTransformSampler;

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformOpenClosedUnit>>
    DistributionSampler<M, R, S, Exponential> for ExponentialInverseTransformSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Lambda(lambda): Lambda,
    ) -> NonNegativeF64 {
        let u01 = UniformOpenClosedUnit::sample_raw(rng, samplers);

        // Inverse transform sample: X = -ln(U(0,1]) / lambda
        -u01.ln::<M>() / lambda
    }
}

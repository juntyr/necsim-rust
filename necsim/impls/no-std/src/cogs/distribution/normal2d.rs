use necsim_core::cogs::{
    distribution::{Normal, Normal2D, RawDistribution, StandardNormal2D},
    DistributionSampler, MathsCore, RngCore,
};

#[allow(clippy::module_name_repetitions)]
pub struct Normal2dSampler;

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, StandardNormal2D>>
    DistributionSampler<M, R, S, Normal2D> for Normal2dSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        Normal { mu, sigma }: Normal,
    ) -> (f64, f64) {
        let (z0, z1) = StandardNormal2D::sample_raw(rng, samplers);

        (z0 * sigma.get() + mu, z1 * sigma.get() + mu)
    }
}

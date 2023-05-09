use necsim_core::cogs::{
    distribution::{
        RawDistribution, StandardNormal2D, UniformClosedOpenUnit, UniformOpenClosedUnit,
    },
    DistributionSampler, MathsCore, RngCore,
};

pub struct StandardNormal2DBoxMullerSampler;

#[allow(clippy::trait_duplication_in_bounds)]
impl<
        M: MathsCore,
        R: RngCore,
        S: DistributionSampler<M, R, S, UniformClosedOpenUnit>
            + DistributionSampler<M, R, S, UniformOpenClosedUnit>,
    > DistributionSampler<M, R, S, StandardNormal2D> for StandardNormal2DBoxMullerSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, samplers: &S, _params: ()) -> (f64, f64) {
        // Basic Box-Muller transform
        let u0 = UniformOpenClosedUnit::sample_raw(rng, samplers);
        let u1 = UniformClosedOpenUnit::sample_raw(rng, samplers);

        let r = M::sqrt(-2.0_f64 * M::ln(u0.get()));
        let theta = -core::f64::consts::TAU * u1.get();

        (r * M::sin(theta), r * M::cos(theta))
    }
}

use necsim_core::cogs::{distribution::Bernoulli, DistributionSampler, MathsCore, RngCore};
use necsim_core_bond::ClosedUnitF64;

#[allow(clippy::module_name_repetitions)]
pub struct Bernoulli64BitSampler;

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, Bernoulli>
    for Bernoulli64BitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, probability: ClosedUnitF64) -> bool {
        #[allow(clippy::cast_precision_loss)]
        const SCALE: f64 = 2.0 * (1u64 << 63) as f64;

        // Safety:
        //  (a) 0 <= probability < 1: probability * SCALE is in [0, 2^64)
        //                            since 1 - 2^-53 is before 1.0
        //  (b) probability == 1    : p_u64 is undefined
        //                            this case is checked for in the return
        let p_u64 = unsafe { (probability.get() * SCALE).to_int_unchecked::<u64>() };

        #[allow(clippy::float_cmp)]
        {
            (rng.sample_u64() < p_u64) || (probability == 1.0_f64)
        }
    }
}

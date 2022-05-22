use necsim_core::cogs::{
    distribution::{UniformClosedOpenUnit, UniformOpenClosedUnit},
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, OpenClosedUnitF64};

#[allow(clippy::module_name_repetitions)]
pub struct Uniform53BitUnitSampler;

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformClosedOpenUnit>
    for Uniform53BitUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> ClosedOpenUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        // Copyright (c) 2014, Taylor R Campbell
        #[allow(clippy::cast_precision_loss)]
        let u01 = ((rng.sample_u64() >> 11) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { ClosedOpenUnitF64::new_unchecked(u01) }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformOpenClosedUnit>
    for Uniform53BitUnitSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> OpenClosedUnitF64 {
        // http://prng.di.unimi.it -> Generating uniform doubles in the unit interval
        // Copyright (c) 2014, Taylor R Campbell
        #[allow(clippy::cast_precision_loss)]
        let u01 =
            (((rng.sample_u64() >> 11) + 1) as f64) * f64::from_bits(0x3CA0_0000_0000_0000_u64); // 0x1.0p-53

        unsafe { OpenClosedUnitF64::new_unchecked(u01) }
    }
}

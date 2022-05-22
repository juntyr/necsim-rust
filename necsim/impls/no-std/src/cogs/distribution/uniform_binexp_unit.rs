use core::{
    convert::TryFrom,
    intrinsics::{likely, unlikely},
};

use necsim_core::cogs::{
    distribution::{UniformClosedOpenUnit, UniformOpenClosedUnit},
    DistributionSampler, MathsCore, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, ClosedUnitF64, OpenClosedUnitF64};

pub struct UniformUnitBinaryExpansionSampler;

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformClosedOpenUnit>
    for UniformUnitBinaryExpansionSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> ClosedOpenUnitF64 {
        loop {
            // Rejection-sample to transform U[0, 1] -> U[0, 1)
            if let Ok(u01) = ClosedOpenUnitF64::try_from(sample_closed_unit_f64(rng)) {
                return u01;
            }
        }
    }
}

impl<M: MathsCore, R: RngCore, S> DistributionSampler<M, R, S, UniformOpenClosedUnit>
    for UniformUnitBinaryExpansionSampler
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(&self, rng: &mut R, _samplers: &S, _params: ()) -> OpenClosedUnitF64 {
        loop {
            // Rejection-sample to transform U[0, 1] -> U(0, 1]
            if let Ok(u01) = OpenClosedUnitF64::try_from(sample_closed_unit_f64(rng)) {
                return u01;
            }
        }
    }
}

// https://prng.di.unimi.it/random_real.c -> random_real
// Copyright (c) 2014, Taylor R Campbell
fn sample_closed_unit_f64<R: RngCore>(rng: &mut R) -> ClosedUnitF64 {
    let mut exponent = -64_i32;
    let mut significand: u64;

    // Read zeros into the exponent until we hit a one;
    //  the rest will go into the significand.
    loop {
        significand = rng.sample_u64();

        if likely(significand != 0) {
            break;
        }

        exponent -= 64;

        // If the exponent falls below -1074 = emin + 1 - p,
        //  the exponent of the smallest subnormal, we are
        //  guaranteed the result will be rounded to zero.
        if unlikely(exponent < -1074) {
            return ClosedUnitF64::zero();
        }
    }

    // There is a 1 somewhere in significand, not necessarily in
    //  the most significant position.
    // If there are leading zeros, shift them into the exponent
    //  and refill the less-significant bits of the significand.
    #[allow(clippy::cast_possible_wrap)]
    let shift = significand.leading_zeros() as i32;

    if shift != 0 {
        exponent -= shift;
        significand <<= shift;
        significand |= rng.sample_u64() >> (64 - shift);
    }

    // Set the sticky bit, since there is almost certainly another 1
    //  in the bit stream.
    // Otherwise, we might round what looks like a tie to even when,
    //  almost certainly, were we to look further in the bit stream,
    //  there would be a 1 breaking the tie.
    significand |= 1;

    // Finally, convert to double (rounding) and scale by 2^exponent.
    #[allow(clippy::cast_precision_loss)]
    let u01 = libm::ldexp(significand as f64, exponent);

    // Safety:
    //  (a) (2^64 - 1) == 2^64 in f64 -> (2^64 - 1) / 2^64 is 1.0
    //  (b) 0 / 2^64 is 0.0
    unsafe { ClosedUnitF64::new_unchecked(u01) }
}

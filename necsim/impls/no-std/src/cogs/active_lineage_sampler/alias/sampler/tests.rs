use alloc::vec::Vec;
use core::num::{NonZeroU128, NonZeroU64, NonZeroUsize};

use necsim_core::cogs::{
    distribution::{IndexU128, IndexU64, IndexUsize, Length, UniformClosedOpenUnit},
    DistributionSampler, Rng, RngCore,
};
use necsim_core_bond::{ClosedOpenUnitF64, NonNegativeF64, PositiveF64};
use necsim_core_maths::MathsCore;

use crate::cogs::{
    distribution::index_from_unit::IndexFromUnitSampler, maths::intrinsics::IntrinsicsMathsCore,
};

use super::{compose_weight, decompose_weight, PositiveF64Decomposed};

#[test]
fn decompose_weights() {
    assert_eq!(
        decompose_weight(PositiveF64::new(1.0_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: 0,
            mantissa: 1_u64 << 52,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(0.125_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: -3,
            mantissa: 1_u64 << 52,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(42.75_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: 5,
            mantissa: 0b1010_1011_u64 << 45,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(f64::from_bits(0x001f_ffff_ffff_ffff_u64)).unwrap()),
        PositiveF64Decomposed {
            exponent: -1022,
            mantissa: 0x001f_ffff_ffff_ffff_u64,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()),
        PositiveF64Decomposed {
            exponent: -1022,
            mantissa: 0x0010_0000_0000_0000_u64,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(f64::from_bits(0x000f_ffff_ffff_ffff_u64)).unwrap()),
        PositiveF64Decomposed {
            exponent: -1023,
            mantissa: 0x001f_ffff_ffff_fffe_u64,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(f64::from_bits(0x0000_0000_ffff_ffff_u64)).unwrap()),
        PositiveF64Decomposed {
            exponent: -1043,
            mantissa: 0x001f_ffff_ffe0_0000_u64,
        }
    );

    assert_eq!(
        decompose_weight(PositiveF64::new(f64::from_bits(0x0000_0000_0000_0001_u64)).unwrap()),
        PositiveF64Decomposed {
            exponent: -1074,
            mantissa: 0x0010_0000_0000_0000_u64,
        }
    );
}

#[test]
fn compose_weights() {
    assert_eq!(compose_weight(42, 0_u128), NonNegativeF64::zero());

    assert_eq!(
        compose_weight(0, 1_u128 << 52),
        PositiveF64::new(1.0_f64).unwrap()
    );

    assert_eq!(
        compose_weight(-3, 1_u128 << 52),
        PositiveF64::new(0.125_f64).unwrap()
    );

    assert_eq!(
        compose_weight(5, 0b1010_1011_u128 << 45),
        PositiveF64::new(42.75_f64).unwrap()
    );

    assert_eq!(
        compose_weight(-1022, 0x001f_ffff_ffff_ffff_u128),
        PositiveF64::new(f64::from_bits(0x001f_ffff_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(-1022, 0x0010_0000_0000_0000_u128),
        PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(-1023, 0x001f_ffff_ffff_fffe_u128),
        PositiveF64::new(f64::from_bits(0x000f_ffff_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(-1043, 0x001f_ffff_ffe0_0000_u128),
        PositiveF64::new(f64::from_bits(0x0000_0000_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(-1074, 0x0010_0000_0000_0000_u128),
        PositiveF64::new(f64::from_bits(0x0000_0000_0000_0001_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(0, (1_u128 << 52) * 8),
        PositiveF64::new(8.0_f64).unwrap()
    );

    assert_eq!(
        compose_weight(0, (1_u128 << 52) * 8 + 3),
        PositiveF64::new(8.0_f64).unwrap()
    );

    assert_eq!(
        compose_weight(0, (1_u128 << 52) * 8 + 4),
        PositiveF64::new(8.000_000_000_000_002_f64).unwrap()
    );

    assert_eq!(
        compose_weight(0, (1_u128 << 52) * 8 + 8),
        PositiveF64::new(8.000_000_000_000_002_f64).unwrap()
    );

    assert_eq!(
        compose_weight(-1023, 0x0010_0000_0000_0000_u128 * 2),
        PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()
    );

    assert_eq!(
        compose_weight(-1023, (0x0000_0000_0000_0001_u128 << 52) * 8),
        compose_weight(-1020, 0x0010_0000_0000_0000_u128)
    );
}

// GRCOV_EXCL_START
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct DummyRng(Vec<f64>);

impl DummyRng {
    pub fn new(mut vec: Vec<f64>) -> Self {
        vec.reverse();

        Self(vec)
    }

    fn sample_f64(&mut self) -> f64 {
        self.0.pop().unwrap()
    }
}

impl RngCore for DummyRng {
    type Seed = [u8; 0];

    #[must_use]
    fn from_seed(_seed: Self::Seed) -> Self {
        Self(Vec::new())
    }

    #[must_use]
    fn sample_u64(&mut self) -> u64 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            ((self.sample_f64() / f64::from_bits(0x3CA0_0000_0000_0000_u64)) as u64) << 11
        }
    }
}

impl Rng<IntrinsicsMathsCore> for DummyRng {
    type Generator = Self;
    type Sampler = DummyDistributionSamplers;

    fn generator(&mut self) -> &mut Self::Generator {
        self
    }

    fn map_generator<F: FnOnce(Self::Generator) -> Self::Generator>(self, map: F) -> Self {
        map(self)
    }

    fn with<F: FnOnce(&mut Self::Generator, &Self::Sampler) -> Q, Q>(&mut self, inner: F) -> Q {
        let samplers = DummyDistributionSamplers {
            index: IndexFromUnitSampler,
        };

        inner(self, &samplers)
    }
}

pub struct DummyDistributionSamplers {
    index: IndexFromUnitSampler,
}

impl<M: MathsCore, S> DistributionSampler<M, DummyRng, S, UniformClosedOpenUnit>
    for DummyDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    #[inline]
    fn sample_distribution(
        &self,
        rng: &mut DummyRng,
        _samplers: &S,
        _params: (),
    ) -> ClosedOpenUnitF64 {
        ClosedOpenUnitF64::new(rng.sample_f64()).unwrap()
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexUsize> for DummyDistributionSamplers
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(
        &self,
        rng: &mut R,
        samplers: &S,
        params: Length<NonZeroUsize>,
    ) -> usize {
        DistributionSampler::<M, R, _, IndexUsize>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU64> for DummyDistributionSamplers
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU64>) -> u64 {
        DistributionSampler::<M, R, _, IndexU64>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}

impl<M: MathsCore, R: RngCore, S: DistributionSampler<M, R, S, UniformClosedOpenUnit>>
    DistributionSampler<M, R, S, IndexU128> for DummyDistributionSamplers
{
    type ConcreteSampler = IndexFromUnitSampler;

    fn concrete(&self) -> &Self::ConcreteSampler {
        &self.index
    }

    #[inline]
    fn sample_distribution(&self, rng: &mut R, samplers: &S, params: Length<NonZeroU128>) -> u128 {
        DistributionSampler::<M, R, _, IndexU128>::sample_distribution(
            &self.index,
            rng,
            samplers,
            params,
        )
    }
}
// GRCOV_EXCL_STOP

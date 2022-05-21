use core::num::{NonZeroU128, NonZeroU64, NonZeroUsize};

use alloc::{vec, vec::Vec};

use necsim_core::cogs::{
    distribution::{IndexU128, IndexU64, IndexUsize, Length},
    Backup, DistributionSampler, Rng, RngCore, SeedableRng,
};
use necsim_core_bond::{NonNegativeF64, PositiveF64};
use necsim_core_maths::MathsCore;

use crate::cogs::{
    maths::intrinsics::IntrinsicsMathsCore,
    rng::{simple::SimpleRng, wyhash::WyHash},
};

use super::{super::decompose_weight, DynamicAliasMethodStackSampler, RejectionSamplingGroup};

#[test]
fn singular_event_group() {
    let group = RejectionSamplingGroup::new(0_u8, 1_u64);

    assert_eq!(&group.events, &[0_u8]);
    assert_eq!(group.iter().copied().collect::<Vec<_>>(), vec![0_u8]);
    assert_eq!(&group.weights, &[1_u64]);
    assert_eq!(group.total_weight, 1_u128);

    assert_eq!(
        group.sample_pop(&mut DummyRng::new(vec![0.0, 0.0])),
        (None, 0_u8)
    );
}

#[test]
fn add_sample_pop_event_group() {
    let mut group = RejectionSamplingGroup::new(0_u8, 1_u64);
    assert_eq!(group.total_weight, 1_u128);

    group.add(1_u8, 2_u64);
    assert_eq!(group.total_weight, 3_u128);

    group.add(2_u8, 3_u64);
    assert_eq!(group.total_weight, 6_u128);

    let (group, sample) = group.sample_pop(&mut DummyRng::new(vec![0.4, 0.0]));
    assert_eq!(sample, 1_u8);
    assert!(group.is_some());
    let mut group = group.unwrap();

    assert_eq!(&group.events, &[0_u8, 2_u8]);
    assert_eq!(group.iter().copied().collect::<Vec<_>>(), vec![0_u8, 2_u8]);
    assert_eq!(&group.weights, &[1_u64, 3_u64]);
    assert_eq!(group.total_weight, 4_u128);

    group.add(3_u8, 4_u64);
    assert_eq!(group.total_weight, 8_u128);

    let (group, sample) = group.sample_pop(&mut DummyRng::new(vec![0.0, 0.0]));
    assert_eq!(sample, 0_u8);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(&group.events, &[3_u8, 2_u8]);
    assert_eq!(group.iter().copied().collect::<Vec<_>>(), vec![3_u8, 2_u8]);
    assert_eq!(&group.weights, &[4_u64, 3_u64]);
    assert_eq!(group.total_weight, 7_u128);

    let (group, sample) = group.sample_pop(&mut DummyRng::new(vec![0.0, 0.0]));
    assert_eq!(sample, 3_u8);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(&group.events, &[2_u8]);
    assert_eq!(group.iter().copied().collect::<Vec<_>>(), vec![2_u8]);
    assert_eq!(&group.weights, &[3_u64]);
    assert_eq!(group.total_weight, 3_u128);

    assert_eq!(
        group.sample_pop(&mut DummyRng::new(vec![0.0, 0.0])),
        (None, 2_u8)
    );
}

#[test]
fn sample_single_group() {
    const N: usize = 10_000_000;

    let mut group = RejectionSamplingGroup::new(
        0_u8,
        decompose_weight(PositiveF64::new(6.0 / 12.0).unwrap()).mantissa,
    );

    for i in 1..6 {
        group.add(
            i,
            decompose_weight(PositiveF64::new(f64::from(6 + i) / 12.0).unwrap()).mantissa,
        );
    }

    assert_eq!(&group.events, &[0, 1, 2, 3, 4, 5]);
    assert_eq!(
        group.iter().copied().collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );

    let mut tally = [0_u64; 6];

    let mut rng = SimpleRng::<IntrinsicsMathsCore, WyHash>::from(WyHash::seed_from_u64(24897));

    for _ in 0..N {
        let (maybe_group, sample) = group.sample_pop(&mut rng);
        group = maybe_group.unwrap();
        group.add(
            sample,
            decompose_weight(PositiveF64::new(f64::from(6 + sample) / 12.0).unwrap()).mantissa,
        );

        tally[sample as usize] += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    for (i, c) in tally.iter().enumerate() {
        let target = (((6 + i) as f64) / 51.0_f64) * 1000.0;
        let measure = ((*c as f64) / (N as f64)) * 1000.0;

        assert!((target - measure).abs() < 2.0);
    }
}

#[test]
fn singular_event_group_full() {
    let mut sampler = DynamicAliasMethodStackSampler::new();
    assert_eq!(sampler.total_weight(), NonNegativeF64::zero());

    sampler.add_push(0_u8, PositiveF64::new(1.0_f64).unwrap());

    assert_eq!(&sampler.exponents, &[0]);
    assert_eq!(
        &sampler.groups,
        &[RejectionSamplingGroup {
            events: alloc::vec![0_u8],
            weights: alloc::vec![1_u64 << 52],
            total_weight: 1_u128 << 52,
        }]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![0_u8]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 1_u128 << 52);
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(1.0_f64).unwrap()
    );

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.0, 0.0, 0.0])),
        Some(0_u8)
    );

    assert_eq!(&sampler.exponents, &[]);
    assert_eq!(&sampler.groups, &[]);
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0_u128);
    assert_eq!(sampler.total_weight(), NonNegativeF64::zero());

    assert_eq!(sampler.sample_pop(&mut DummyRng::new(vec![])), None);
}

#[test]
#[allow(clippy::too_many_lines)]
fn add_remove_event_full() {
    let mut sampler = DynamicAliasMethodStackSampler::default();
    assert_eq!(sampler.total_weight(), NonNegativeF64::zero());
    sampler.add_push(0_u8, PositiveF64::new(1.0_f64).unwrap());
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(1.0_f64).unwrap()
    );
    sampler.add_push(1_u8, PositiveF64::new(1.5_f64).unwrap());
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(2.5_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[0]);
    assert_eq!(
        &sampler.groups,
        &[RejectionSamplingGroup {
            events: alloc::vec![0_u8, 1_u8],
            weights: alloc::vec![1_u64 << 52, 3_u64 << 51],
            total_weight: 5_u128 << 51,
        }]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![0_u8, 1_u8]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 5_u128 << 51);

    sampler.add_push(2_u8, PositiveF64::new(0.125_f64).unwrap());
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(2.625_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[0, -3]);
    assert_eq!(
        &sampler.groups,
        &[
            RejectionSamplingGroup {
                events: alloc::vec![0_u8, 1_u8],
                weights: alloc::vec![1_u64 << 52, 3_u64 << 51],
                total_weight: 5_u128 << 51,
            },
            RejectionSamplingGroup {
                events: alloc::vec![2_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            }
        ]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![0_u8, 1_u8, 2_u8]
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b1_0101_u128 << 52);

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.0, 0.9, 0.0])),
        Some(1_u8)
    );
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(1.125_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[0, -3]);
    assert_eq!(
        &sampler.groups,
        &[
            RejectionSamplingGroup {
                events: alloc::vec![0_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            },
            RejectionSamplingGroup {
                events: alloc::vec![2_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            }
        ]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![0_u8, 2_u8]
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b1001_u128 << 52);

    sampler.add_push(3_u8, PositiveF64::new(12.0_f64).unwrap());
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(13.125_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[3, 0, -3]);
    assert_eq!(
        &sampler.groups,
        &[
            RejectionSamplingGroup {
                events: alloc::vec![3_u8],
                weights: alloc::vec![3_u64 << 51],
                total_weight: 3_u128 << 51,
            },
            RejectionSamplingGroup {
                events: alloc::vec![0_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            },
            RejectionSamplingGroup {
                events: alloc::vec![2_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            }
        ]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![3_u8, 0_u8, 2_u8]
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b0110_1001_u128 << 52);

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.991, 0.0, 0.0])),
        Some(2_u8)
    );
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(13.0_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[3, 0]);
    assert_eq!(
        &sampler.groups,
        &[
            RejectionSamplingGroup {
                events: alloc::vec![3_u8],
                weights: alloc::vec![3_u64 << 51],
                total_weight: 3_u128 << 51,
            },
            RejectionSamplingGroup {
                events: alloc::vec![0_u8],
                weights: alloc::vec![1_u64 << 52],
                total_weight: 1_u128 << 52,
            }
        ]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![3_u8, 0_u8]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0b1101_u128 << 52);

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.95, 0.0, 0.0])),
        Some(0_u8)
    );
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(12.0_f64).unwrap()
    );

    assert_eq!(&sampler.exponents, &[3]);
    assert_eq!(
        &sampler.groups,
        &[RejectionSamplingGroup {
            events: alloc::vec![3_u8],
            weights: alloc::vec![3_u64 << 51],
            total_weight: 3_u128 << 51,
        }]
    );
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![3_u8]
    );
    assert_eq!(sampler.min_exponent, 3);
    assert_eq!(sampler.total_weight, 3 << 51);

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.0, 0.0, 0.0])),
        Some(3_u8)
    );
    assert_eq!(sampler.total_weight(), NonNegativeF64::zero());

    assert_eq!(&sampler.exponents, &[]);
    assert_eq!(&sampler.groups, &[]);
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0);

    assert_eq!(sampler.sample_pop(&mut DummyRng::new(vec![])), None);
}

#[test]
fn sample_single_group_full() {
    const N: usize = 10_000_000;

    let mut rng = SimpleRng::<IntrinsicsMathsCore, WyHash>::from(WyHash::seed_from_u64(471_093));

    let mut sampler = DynamicAliasMethodStackSampler::with_capacity(6);

    assert!(sampler.sample_pop(&mut rng).is_none());

    for i in 0..6_u8 {
        sampler.add_push(i, PositiveF64::new(f64::from(6 + i) / 12.0).unwrap());
    }

    assert_eq!(&sampler.exponents, &[-1]);
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![0, 1, 2, 3, 4, 5]
    );
    assert_eq!(sampler.min_exponent, -1);
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(4.25_f64).unwrap()
    );

    let mut tally = [0_u64; 6];

    for _ in 0..N {
        let sample = sampler.sample_pop(&mut rng).unwrap();
        sampler.add_push(
            sample,
            PositiveF64::new(f64::from(6 + sample) / 12.0).unwrap(),
        );

        tally[sample as usize] += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    for (i, c) in tally.iter().enumerate() {
        let target = (((6 + i) as f64) / 51.0_f64) * 1000.0;
        let measure = ((*c as f64) / (N as f64)) * 1000.0;

        assert!((target - measure).abs() < 2.0);
    }
}

#[test]
fn sample_three_groups_full() {
    const N: usize = 10_000_000;

    let mut rng = SimpleRng::<IntrinsicsMathsCore, WyHash>::from(WyHash::seed_from_u64(739_139));

    let mut sampler = DynamicAliasMethodStackSampler::with_capacity(6);

    assert!(sampler.sample_pop(&mut rng).is_none());

    for i in 1..=6_u8 {
        sampler.add_push(i, PositiveF64::new(f64::from(i)).unwrap());
    }

    assert_eq!(&sampler.exponents, &[2, 1, 0]);
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![4, 5, 6, 2, 3, 1]
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(
        sampler.total_weight(),
        NonNegativeF64::new(21.0_f64).unwrap()
    );

    let mut tally = [0_u64; 6];

    for _ in 0..N {
        let sample = sampler.sample_pop(&mut rng).unwrap();
        sampler.add_push(sample, PositiveF64::new(f64::from(sample)).unwrap());

        tally[sample as usize - 1] += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    for (i, c) in tally.iter().enumerate() {
        let target = (((i + 1) as f64) / 21.0_f64) * 1000.0;
        let measure = ((*c as f64) / (N as f64)) * 1000.0;

        assert!((target - measure).abs() < 2.0);
    }
}

#[test]
fn sample_three_groups_full_reverse() {
    const N: usize = 10_000_000;

    let mut rng = SimpleRng::<IntrinsicsMathsCore, WyHash>::from(WyHash::seed_from_u64(248_971));

    let mut sampler = DynamicAliasMethodStackSampler::with_capacity(6);

    assert!(sampler.sample_pop(&mut rng).is_none());

    for i in (1..=6_u8).rev() {
        sampler.add_push(i, PositiveF64::new(f64::from(i)).unwrap());
    }

    assert_eq!(&sampler.exponents, &[2, 1, 0]);
    assert_eq!(
        sampler
            .iter_all_events_ordered()
            .copied()
            .collect::<Vec<_>>(),
        vec![6, 5, 4, 3, 2, 1]
    );
    assert_eq!(sampler.min_exponent, 0);

    let mut tally = [0_u64; 6];

    for _ in 0..N {
        let sample = sampler.sample_pop(&mut rng).unwrap();
        sampler.add_push(sample, PositiveF64::new(f64::from(sample)).unwrap());

        tally[sample as usize - 1] += 1;
    }

    #[allow(clippy::cast_precision_loss)]
    for (i, c) in tally.iter().enumerate() {
        let target = (((i + 1) as f64) / 21.0_f64) * 1000.0;
        let measure = ((*c as f64) / (N as f64)) * 1000.0;

        assert!((target - measure).abs() < 2.0);
    }
}

#[test]
fn debug_display_sampler() {
    let mut sampler = DynamicAliasMethodStackSampler::with_capacity(6);

    assert_eq!(
        &alloc::format!("{sampler:?}"),
        "DynamicAliasMethodStackSampler { exponents: [], total_weight: 0.0 }"
    );

    for i in (1..=6_u8).rev() {
        sampler.add_push(i, PositiveF64::new(f64::from(i)).unwrap());
    }

    assert_eq!(
        &alloc::format!("{sampler:?}"),
        "DynamicAliasMethodStackSampler { exponents: [2, 1, 0], total_weight: 21.0 }"
    );

    let mut sampler_clone = unsafe { sampler.backup_unchecked() };

    assert_eq!(
        sampler.sample_pop(&mut DummyRng::new(vec![0.75, 0.0, 0.0])),
        Some(3_u8)
    );
    assert_eq!(
        sampler_clone.sample_pop(&mut DummyRng::new(vec![0.99, 0.0, 0.0])),
        Some(1_u8)
    );

    assert_eq!(
        &alloc::format!("{sampler:?}"),
        "DynamicAliasMethodStackSampler { exponents: [2, 1, 0], total_weight: 18.0 }"
    );
    assert_eq!(
        &alloc::format!("{sampler_clone:?}"),
        "DynamicAliasMethodStackSampler { exponents: [2, 1], total_weight: 20.0 }"
    );
}

// GRCOV_EXCL_START
#[derive(Debug, serde::Serialize, serde::Deserialize)]
struct DummyRng(Vec<f64>);

impl DummyRng {
    fn new(mut vec: Vec<f64>) -> Self {
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
        let samplers = DummyDistributionSamplers;

        inner(self, &samplers)
    }
}

struct DummyDistributionSamplers;

impl DistributionSampler<IntrinsicsMathsCore, DummyRng, DummyDistributionSamplers, IndexUsize>
    for DummyDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut DummyRng,
        _samplers: &DummyDistributionSamplers,
        Length(length): Length<NonZeroUsize>,
    ) -> usize {
        let u01 = rng.sample_f64();

        // Safety: U[0, 1) * length in [0, 2^[32/64]) is a valid [u32/u64]
        //         since (1 - 2^-53) * 2^[32/64] <= (2^[32/64] - 1)
        #[allow(clippy::cast_precision_loss)]
        let index = unsafe {
            IntrinsicsMathsCore::floor(u01 * (length.get() as f64)).to_int_unchecked::<usize>()
        };

        if cfg!(target_pointer_width = "32") {
            // Note: [0, 2^32) is losslessly represented in f64
            index
        } else {
            // Note: Ensure index < length despite
            //       usize->f64->usize precision loss
            index.min(length.get() - 1)
        }
    }
}

impl DistributionSampler<IntrinsicsMathsCore, DummyRng, DummyDistributionSamplers, IndexU64>
    for DummyDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut DummyRng,
        _samplers: &DummyDistributionSamplers,
        Length(length): Length<NonZeroU64>,
    ) -> u64 {
        let u01 = rng.sample_f64();

        // Safety: U[0, 1) * length in [0, 2^64) is a valid u64
        //         since (1 - 2^-53) * 2^64 <= (2^64 - 1)
        #[allow(clippy::cast_precision_loss)]
        let index = unsafe {
            IntrinsicsMathsCore::floor(u01 * (length.get() as f64)).to_int_unchecked::<u64>()
        };

        // Note: Ensure index < length despite u64->f64->u64 precision loss
        index.min(length.get() - 1)
    }
}

impl DistributionSampler<IntrinsicsMathsCore, DummyRng, DummyDistributionSamplers, IndexU128>
    for DummyDistributionSamplers
{
    type ConcreteSampler = Self;

    fn concrete(&self) -> &Self::ConcreteSampler {
        self
    }

    fn sample_distribution(
        &self,
        rng: &mut DummyRng,
        _samplers: &DummyDistributionSamplers,
        Length(length): Length<NonZeroU128>,
    ) -> u128 {
        let u01 = rng.sample_f64();

        // Safety: U[0, 1) * length in [0, 2^128) is a valid u128
        //         since (1 - 2^-53) * 2^128 <= (2^128 - 1)
        #[allow(clippy::cast_precision_loss)]
        let index = unsafe {
            IntrinsicsMathsCore::floor(u01 * (length.get() as f64)).to_int_unchecked::<u128>()
        };

        // Note: Ensure index < length despite u128->f64->u128 precision loss
        index.min(length.get() - 1)
    }
}
// GRCOV_EXCL_STOP

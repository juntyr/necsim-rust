use hashbrown::HashMap;

use necsim_core::cogs::SeedableRng;
use necsim_core_bond::PositiveF64;
use necsim_core_maths::IntrinsicsMathsCore;

use crate::cogs::rng::wyhash::WyHash;

use super::{
    DynamicAliasMethodSampler, EventLocation, PositiveF64Decomposed, RejectionSamplingGroup,
};

#[test]
fn decompose_weights() {
    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(PositiveF64::new(1.0_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: 0,
            mantissa: 1_u64 << 52,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(PositiveF64::new(0.125_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: -3,
            mantissa: 1_u64 << 52,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(PositiveF64::new(42.75_f64).unwrap()),
        PositiveF64Decomposed {
            exponent: 5,
            mantissa: 0b1010_1011_u64 << 45,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(
            PositiveF64::new(f64::from_bits(0x001f_ffff_ffff_ffff_u64)).unwrap()
        ),
        PositiveF64Decomposed {
            exponent: -1022,
            mantissa: 0x001f_ffff_ffff_ffff_u64,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(
            PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()
        ),
        PositiveF64Decomposed {
            exponent: -1022,
            mantissa: 0x0010_0000_0000_0000_u64,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(
            PositiveF64::new(f64::from_bits(0x000f_ffff_ffff_ffff_u64)).unwrap()
        ),
        PositiveF64Decomposed {
            exponent: -1023,
            mantissa: 0x001f_ffff_ffff_fffe_u64,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(
            PositiveF64::new(f64::from_bits(0x0000_0000_ffff_ffff_u64)).unwrap()
        ),
        PositiveF64Decomposed {
            exponent: -1043,
            mantissa: 0x001f_ffff_ffe0_0000_u64,
        }
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::decompose_weight(
            PositiveF64::new(f64::from_bits(0x0000_0000_0000_0001_u64)).unwrap()
        ),
        PositiveF64Decomposed {
            exponent: -1074,
            mantissa: 0x0010_0000_0000_0000_u64,
        }
    );
}

#[test]
fn compose_weights() {
    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(0, 1_u128 << 52),
        PositiveF64::new(1.0_f64).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-3, 1_u128 << 52),
        PositiveF64::new(0.125_f64).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(5, 0b1010_1011_u128 << 45),
        PositiveF64::new(42.75_f64).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1022, 0x001f_ffff_ffff_ffff_u128),
        PositiveF64::new(f64::from_bits(0x001f_ffff_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1022, 0x0010_0000_0000_0000_u128),
        PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1023, 0x001f_ffff_ffff_fffe_u128),
        PositiveF64::new(f64::from_bits(0x000f_ffff_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1043, 0x001f_ffff_ffe0_0000_u128),
        PositiveF64::new(f64::from_bits(0x0000_0000_ffff_ffff_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1074, 0x0010_0000_0000_0000_u128),
        PositiveF64::new(f64::from_bits(0x0000_0000_0000_0001_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(0, (1_u128 << 52) * 8),
        PositiveF64::new(8.0_f64).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(-1023, 0x0010_0000_0000_0000_u128 * 2),
        PositiveF64::new(f64::from_bits(0x0010_0000_0000_0000_u64)).unwrap()
    );

    assert_eq!(
        DynamicAliasMethodSampler::<u8>::compose_weight(
            -1023,
            (0x0000_0000_0000_0001_u128 << 52) * 8
        ),
        DynamicAliasMethodSampler::<u8>::compose_weight(-1020, 0x0010_0000_0000_0000_u128)
    );
}

#[test]
fn singular_event_group() {
    let group = RejectionSamplingGroup::new(0_u8, 1_u64);

    assert_eq!(&group.events, &[0_u8]);
    assert_eq!(&group.weights, &[1_u64]);
    assert_eq!(group.total_weight, 1_u128);

    assert!(group.remove(0, &mut HashMap::new()).is_none());
}

#[test]
fn add_remove_event_group() {
    let mut group = RejectionSamplingGroup::new(0_u8, 1_u64);

    let mut lookup = HashMap::new();
    lookup.insert(
        0_u8,
        EventLocation {
            exponent: 7,
            group_index: 0,
        },
    );

    assert_eq!(group.add(1_u8, 2_u64), 1_usize);
    assert_eq!(group.total_weight, 3_u128);
    lookup.insert(
        1_u8,
        EventLocation {
            exponent: 42,
            group_index: 1,
        },
    );

    assert_eq!(group.add(2_u8, 3_u64), 2_usize);
    assert_eq!(group.total_weight, 6_u128);
    lookup.insert(
        2_u8,
        EventLocation {
            exponent: 9,
            group_index: 2,
        },
    );

    let group = group.remove(1, &mut lookup);
    assert!(group.is_some());
    let mut group = group.unwrap();

    assert_eq!(&group.events, &[0_u8, 2_u8]);
    assert_eq!(&group.weights, &[1_u64, 3_u64]);
    assert_eq!(group.total_weight, 4_u128);
    assert_eq!(
        lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 7,
            group_index: 0
        })
        .as_ref()
    );
    assert_eq!(
        lookup.get(&2_u8),
        Some(EventLocation {
            exponent: 9,
            group_index: 1
        })
        .as_ref()
    );

    assert_eq!(group.add(3_u8, 4_u64), 2_usize);
    assert_eq!(group.total_weight, 8_u128);
    lookup.insert(
        3_u8,
        EventLocation {
            exponent: 71,
            group_index: 2,
        },
    );

    let group = group.remove(0, &mut lookup);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(&group.events, &[3_u8, 2_u8]);
    assert_eq!(&group.weights, &[4_u64, 3_u64]);
    assert_eq!(group.total_weight, 7_u128);
    assert_eq!(
        lookup.get(&2_u8),
        Some(EventLocation {
            exponent: 9,
            group_index: 1
        })
        .as_ref()
    );
    assert_eq!(
        lookup.get(&3_u8),
        Some(EventLocation {
            exponent: 71,
            group_index: 0
        })
        .as_ref()
    );

    let group = group.remove(0, &mut lookup);
    assert!(group.is_some());
    let group = group.unwrap();

    assert_eq!(&group.events, &[2_u8]);
    assert_eq!(&group.weights, &[3_u64]);
    assert_eq!(group.total_weight, 3_u128);
    assert_eq!(
        lookup.get(&2_u8),
        Some(EventLocation {
            exponent: 9,
            group_index: 0
        })
        .as_ref()
    );

    assert!(group.remove(0, &mut HashMap::new()).is_none());
}

#[test]
fn sample_single_group() {
    const N: usize = 10_000_000;

    let mut group = RejectionSamplingGroup::new(
        0_u8,
        DynamicAliasMethodSampler::<u8>::decompose_weight(PositiveF64::new(6.0 / 12.0).unwrap())
            .mantissa,
    );

    for i in 1..6 {
        assert_eq!(
            group.add(
                i,
                DynamicAliasMethodSampler::<u8>::decompose_weight(
                    PositiveF64::new(f64::from(6 + i) / 12.0).unwrap()
                )
                .mantissa
            ),
            i as usize
        );
    }

    assert_eq!(&group.events, &[0, 1, 2, 3, 4, 5]);

    let mut tally = [0_u64; 6];

    let mut rng = WyHash::<IntrinsicsMathsCore>::seed_from_u64(24897);

    for _ in 0..N {
        tally[*group.sample(&mut rng) as usize] += 1;
    }

    assert_eq!(
        tally
            .iter()
            .map(|c| (((*c as f64) / (N as f64)) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
        (0..6)
            .map(|i| ((f64::from(6 + i) / 51.0_f64) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
    );
}

#[test]
fn singular_event_group_full() {
    let mut sampler = DynamicAliasMethodSampler::new();
    sampler.add(0_u8, PositiveF64::new(1.0_f64).unwrap());

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 1_u128 << 52);

    sampler.remove(&0_u8);

    assert_eq!(&sampler.exponents, &[]);
    assert_eq!(&sampler.groups, &[]);
    assert_eq!(sampler.lookup.get(&0_u8), None);
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0_u128);
}

#[test]
#[allow(clippy::too_many_lines)]
fn add_remove_event_full() {
    let mut sampler = DynamicAliasMethodSampler::new();
    sampler.add(0_u8, PositiveF64::new(1.0_f64).unwrap());
    sampler.add(1_u8, PositiveF64::new(1.5_f64).unwrap());

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(
        sampler.lookup.get(&1_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 1,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 5_u128 << 51);

    sampler.add(2_u8, PositiveF64::new(0.125_f64).unwrap());

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(
        sampler.lookup.get(&1_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 1,
        })
        .as_ref()
    );
    assert_eq!(
        sampler.lookup.get(&2_u8),
        Some(EventLocation {
            exponent: -3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b1_0101_u128 << 52);

    sampler.remove(&1_u8);

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.lookup.get(&1_u8), None);
    assert_eq!(
        sampler.lookup.get(&2_u8),
        Some(EventLocation {
            exponent: -3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b1001_u128 << 52);

    sampler.add(3_u8, PositiveF64::new(12.0_f64).unwrap());

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(
        sampler.lookup.get(&2_u8),
        Some(EventLocation {
            exponent: -3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(
        sampler.lookup.get(&3_u8),
        Some(EventLocation {
            exponent: 3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, -3);
    assert_eq!(sampler.total_weight, 0b0110_1001_u128 << 52);

    sampler.remove(&2_u8);

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
        sampler.lookup.get(&0_u8),
        Some(EventLocation {
            exponent: 0,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.lookup.get(&2_u8), None);
    assert_eq!(
        sampler.lookup.get(&3_u8),
        Some(EventLocation {
            exponent: 3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0b1101_u128 << 52);

    sampler.remove(&0_u8);

    assert_eq!(&sampler.exponents, &[3]);
    assert_eq!(
        &sampler.groups,
        &[RejectionSamplingGroup {
            events: alloc::vec![3_u8],
            weights: alloc::vec![3_u64 << 51],
            total_weight: 3_u128 << 51,
        }]
    );
    assert_eq!(sampler.lookup.get(&0_u8), None);
    assert_eq!(
        sampler.lookup.get(&3_u8),
        Some(EventLocation {
            exponent: 3,
            group_index: 0,
        })
        .as_ref()
    );
    assert_eq!(sampler.min_exponent, 3);
    assert_eq!(sampler.total_weight, 3 << 51);

    sampler.remove(&3_u8);

    assert_eq!(&sampler.exponents, &[]);
    assert_eq!(&sampler.groups, &[]);
    assert_eq!(sampler.lookup.get(&3_u8), None);
    assert_eq!(sampler.min_exponent, 0);
    assert_eq!(sampler.total_weight, 0);
}

#[test]
fn sample_single_group_full() {
    const N: usize = 10_000_000;

    let mut sampler = DynamicAliasMethodSampler::new();

    for i in 0..6_u8 {
        sampler.add(i, PositiveF64::new(f64::from(6 + i) / 12.0).unwrap());
    }

    assert_eq!(&sampler.exponents, &[-1]);
    assert_eq!(sampler.min_exponent, -1);

    let mut tally = [0_u64; 6];

    let mut rng = WyHash::<IntrinsicsMathsCore>::seed_from_u64(24897);

    for _ in 0..N {
        tally[*sampler.sample(&mut rng).unwrap() as usize] += 1;
    }

    assert_eq!(
        tally
            .iter()
            .map(|c| (((*c as f64) / (N as f64)) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
        (0..6)
            .map(|i| ((f64::from(6 + i) / 51.0_f64) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
    );
}

#[test]
fn sample_three_groups_full() {
    const N: usize = 100_000_000;

    let mut sampler = DynamicAliasMethodSampler::new();

    for i in 1..=6_u8 {
        sampler.add(i, PositiveF64::new(f64::from(i)).unwrap());
    }

    assert_eq!(&sampler.exponents, &[2, 1, 0]);
    assert_eq!(sampler.min_exponent, 0);

    let mut tally = [0_u64; 6];

    let mut rng = WyHash::<IntrinsicsMathsCore>::seed_from_u64(24897);

    for _ in 0..N {
        tally[*sampler.sample(&mut rng).unwrap() as usize - 1] += 1;
    }

    assert_eq!(
        tally
            .iter()
            .map(|c| (((*c as f64) / (N as f64)) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
        (1..=6)
            .map(|i| ((f64::from(i) / 21.0_f64) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
    );
}

#[test]
fn sample_three_groups_full_reverse() {
    const N: usize = 100_000_000;

    let mut sampler = DynamicAliasMethodSampler::new();

    for i in (1..=6_u8).rev() {
        sampler.add(i, PositiveF64::new(f64::from(i)).unwrap());
    }

    assert_eq!(&sampler.exponents, &[2, 1, 0]);
    assert_eq!(sampler.min_exponent, 0);

    let mut tally = [0_u64; 6];

    let mut rng = WyHash::<IntrinsicsMathsCore>::seed_from_u64(24897);

    for _ in 0..N {
        tally[*sampler.sample(&mut rng).unwrap() as usize - 1] += 1;
    }

    assert_eq!(
        tally
            .iter()
            .map(|c| (((*c as f64) / (N as f64)) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
        (1..=6)
            .map(|i| ((f64::from(i) / 21.0_f64) * 1000.0).round() as u64)
            .collect::<alloc::vec::Vec<_>>(),
    );
}

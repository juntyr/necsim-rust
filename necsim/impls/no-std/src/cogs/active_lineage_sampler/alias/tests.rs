use hashbrown::HashMap;
use necsim_core::cogs::RngCore;
use necsim_core_bond::PositiveF64;

use crate::cogs::{active_lineage_sampler::alias::EventLocation, rng::wyhash::WyHash};

use super::{DynamicAliasMethodSampler, RejectionSamplingGroup};

#[test]
fn singular_event_group() {
    let group = RejectionSamplingGroup::new(0_u8, 1_u64);

    assert_eq!(&group.events, &[0_u8]);
    assert_eq!(&group.weights, &[1_u64]);
    assert_eq!(group.total_weight, 1_u128);

    assert!(group.remove(0, &mut HashMap::new()).is_none());
}

#[test]
fn two_event_group() {
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

    let mut rng = WyHash::seed_from_u64(24897);

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

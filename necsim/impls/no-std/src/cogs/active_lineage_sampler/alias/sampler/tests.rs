use necsim_core_bond::{NonNegativeF64, PositiveF64};

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

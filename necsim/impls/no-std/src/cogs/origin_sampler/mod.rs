pub mod almost_infinite;
pub mod in_memory;
pub mod non_spatial;
pub mod percentage;
pub mod spatially_implicit;
pub mod uniform_partition;

const INV_PHI: f64 = 6.180_339_887_498_949e-1_f64;

pub fn sample_all() -> impl Iterator<Item = u64> {
    0..=u64::MAX
}

pub fn sample_percentage(
    iter: impl Iterator<Item = u64>,
    percentage: f64,
) -> impl Iterator<Item = u64> {
    debug_assert!(
        (0.0_f64..=1.0_f64).contains(&percentage),
        "percentage is in [0, 1]"
    );

    iter.scan(0.5_f64, move |quasi_random, index| {
        *quasi_random = necsim_core::intrinsics::fract(*quasi_random + INV_PHI);

        Some(if *quasi_random < percentage {
            Some(index)
        } else {
            None
        })
    })
    .flatten()
}

pub fn sample_partition(
    mut iter: impl Iterator<Item = u64>,
    offset: u32,
    stride: u32,
) -> impl Iterator<Item = u64> {
    let _ = iter.advance_by(offset as usize);

    iter.step_by(stride as usize)
}

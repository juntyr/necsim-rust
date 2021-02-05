use core::{
    fmt,
    ops::{Deref, DerefMut, RangeFrom},
};

const INV_PHI: f64 = 6.180_339_887_498_949e-1_f64;

#[allow(clippy::module_name_repetitions)]
pub struct OriginPreSampler<I: Iterator<Item = u64>>(I);

impl<I: Iterator<Item = u64>> Deref for OriginPreSampler<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl<I: Iterator<Item = u64>> DerefMut for OriginPreSampler<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl<I: Iterator<Item = u64>> fmt::Debug for OriginPreSampler<I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("OriginPreSampler").finish()
    }
}

impl OriginPreSampler<RangeFrom<u64>> {
    #[must_use]
    pub fn all() -> Self {
        Self(0..)
    }
}

impl<I: Iterator<Item = u64>> OriginPreSampler<I> {
    #[must_use]
    pub fn percentage(mut self, percentage: f64) -> OriginPreSampler<impl Iterator<Item = u64>> {
        use necsim_core::intrinsics::{floor, ln};

        debug_assert!(
            (0.0_f64..=1.0_f64).contains(&percentage),
            "percentage is in [0, 1]"
        );

        let inv_geometric_sample_rate = ln(1.0_f64 - percentage).recip();

        OriginPreSampler(
            core::iter::repeat(()).scan(0.5_f64, move |quasi_random, _| {
                *quasi_random = necsim_core::intrinsics::fract(*quasi_random + INV_PHI);

                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let skip =
                    floor(necsim_core::intrinsics::ln(*quasi_random) * inv_geometric_sample_rate)
                        as usize;

                self.nth(skip)
            }),
        )
    }

    pub fn partition(
        mut self,
        offset: u32,
        stride: u32,
    ) -> OriginPreSampler<impl Iterator<Item = u64>> {
        let _ = self.advance_by(offset as usize);

        OriginPreSampler(self.0.step_by(stride as usize))
    }
}

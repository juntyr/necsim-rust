use core::{
    fmt,
    iter::Empty,
    ops::{Deref, DerefMut, RangeFrom},
};

use necsim_core_bond::ClosedUnitF64;
use necsim_partitioning_core::partition::Partition;

const INV_PHI: f64 = 6.180_339_887_498_949e-1_f64;

#[allow(clippy::module_name_repetitions)]
pub struct OriginPreSampler<I: Iterator<Item = u64>> {
    inner: I,
    proportion: ClosedUnitF64,
}

impl<I: Iterator<Item = u64>> OriginPreSampler<I> {
    pub fn get_sample_proportion(&self) -> ClosedUnitF64 {
        self.proportion
    }
}

impl<I: Iterator<Item = u64>> Deref for OriginPreSampler<I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<I: Iterator<Item = u64>> DerefMut for OriginPreSampler<I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<I: Iterator<Item = u64>> fmt::Debug for OriginPreSampler<I> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(OriginPreSampler))
            .field("proportion", &self.proportion)
            .finish()
    }
}

impl OriginPreSampler<RangeFrom<u64>> {
    #[must_use]
    pub fn all() -> Self {
        Self {
            inner: 0..,
            proportion: ClosedUnitF64::one(),
        }
    }
}

impl OriginPreSampler<Empty<u64>> {
    #[must_use]
    pub fn none() -> Self {
        Self {
            inner: core::iter::empty(),
            proportion: ClosedUnitF64::zero(),
        }
    }
}

impl<I: Iterator<Item = u64>> OriginPreSampler<I> {
    #[must_use]
    pub fn percentage(
        mut self,
        percentage: ClosedUnitF64,
    ) -> OriginPreSampler<impl Iterator<Item = u64>> {
        let inv_geometric_sample_rate = libm::log(1.0_f64 - percentage.get()).recip();

        OriginPreSampler {
            proportion: self.proportion * percentage,
            inner: core::iter::repeat(()).scan(0.5_f64, move |quasi_random, _| {
                if percentage <= 0.0_f64 {
                    return None;
                }

                if percentage >= 1.0_f64 {
                    return self.next();
                }

                // q = (q + INV_PHI) % 1  where q >= 0
                *quasi_random += INV_PHI;
                *quasi_random -= if *quasi_random >= 1.0_f64 {
                    1.0_f64
                } else {
                    0.0_f64
                };

                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let skip = (libm::log(*quasi_random) * inv_geometric_sample_rate) as usize;

                self.nth(skip)
            }),
        }
    }

    pub fn partition(
        mut self,
        partition: Partition,
    ) -> OriginPreSampler<impl Iterator<Item = u64>> {
        let _ = self.advance_by(partition.rank() as usize);

        OriginPreSampler {
            proportion: self.proportion / partition.size(),
            inner: self.inner.step_by(partition.size().get() as usize),
        }
    }
}

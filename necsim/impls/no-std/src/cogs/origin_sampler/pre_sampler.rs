use core::{
    fmt,
    marker::PhantomData,
    ops::{Deref, DerefMut, RangeFrom},
};

use necsim_core::cogs::MathsCore;

const INV_PHI: f64 = 6.180_339_887_498_949e-1_f64;

#[allow(clippy::module_name_repetitions)]
pub struct OriginPreSampler<M: MathsCore, I: Iterator<Item = u64>> {
    inner: I,
    proportion: f64,
    _marker: PhantomData<M>,
}

impl<M: MathsCore, I: Iterator<Item = u64>> OriginPreSampler<M, I> {
    #[debug_ensures((0.0_f64..=1.0_f64).contains(&ret), "returns a proportion")]
    pub fn get_sample_proportion(&self) -> f64 {
        self.proportion
    }
}

impl<M: MathsCore, I: Iterator<Item = u64>> Deref for OriginPreSampler<M, I> {
    type Target = I;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<M: MathsCore, I: Iterator<Item = u64>> DerefMut for OriginPreSampler<M, I> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<M: MathsCore, I: Iterator<Item = u64>> fmt::Debug for OriginPreSampler<M, I> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        f.debug_struct(stringify!(OriginPreSampler))
            .field("proportion", &self.proportion)
            .finish()
    }
}

impl<M: MathsCore> OriginPreSampler<M, RangeFrom<u64>> {
    #[must_use]
    pub fn all() -> Self {
        Self {
            inner: 0..,
            proportion: 1.0_f64,
            _marker: PhantomData::<M>,
        }
    }
}

impl<M: MathsCore, I: Iterator<Item = u64>> OriginPreSampler<M, I> {
    #[must_use]
    #[debug_requires((0.0_f64..=1.0_f64).contains(&percentage), "percentage is in [0, 1]")]
    pub fn percentage(mut self, percentage: f64) -> OriginPreSampler<M, impl Iterator<Item = u64>> {
        let inv_geometric_sample_rate = M::ln(1.0_f64 - percentage).recip();

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
                *quasi_random -= M::floor(*quasi_random);

                #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
                let skip = M::floor(M::ln(*quasi_random) * inv_geometric_sample_rate) as usize;

                self.nth(skip)
            }),
            _marker: PhantomData::<M>,
        }
    }

    pub fn partition(
        mut self,
        offset: u32,
        stride: u32,
    ) -> OriginPreSampler<M, impl Iterator<Item = u64>> {
        let _ = self.advance_by(offset as usize);

        OriginPreSampler {
            proportion: self.proportion / f64::from(stride),
            inner: self.inner.step_by(stride as usize),
            _marker: PhantomData::<M>,
        }
    }
}

use core::{iter::Iterator, marker::PhantomData};

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    intrinsics::{ceil, fract, sqrt},
    landscape::IndexedLocation,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct PercentageOriginSampler<'h, H: Habitat> {
    base_sampler: H::OriginSampler<'h>,
    sample_percentage: f64,
    counter: u64,
    inv_phi: f64,
    _marker: PhantomData<&'h H>,
}

impl<'h, H: Habitat> PercentageOriginSampler<'h, H> {
    #[must_use]
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&sample_percentage),
        "sample_percentage is a percentage"
    )]
    pub fn new(base_sampler: H::OriginSampler<'h>, sample_percentage: f64) -> Self {
        Self {
            base_sampler,
            sample_percentage,
            counter: 0,
            inv_phi: ((1.0_f64 + sqrt(5.0_f64)) * 0.5_f64).recip(),
            _marker: PhantomData::<&'h H>,
        }
    }
}

#[contract_trait]
impl<'h, H: Habitat> OriginSampler<'h, H> for PercentageOriginSampler<'h, H> {
    fn habitat(&self) -> &'h H {
        self.base_sampler.habitat()
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let upper_bound_size_hint =
            ceil((self.base_sampler.full_upper_bound_size_hint() as f64) * self.sample_percentage)
                as u64;

        upper_bound_size_hint
    }
}

impl<'h, H: Habitat> Iterator for PercentageOriginSampler<'h, H> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(next) = self.base_sampler.next() {
            self.counter += 1;

            #[allow(clippy::cast_precision_loss)]
            let quasi_random_uniform = fract(0.5_f64 + (self.counter as f64) * self.inv_phi);

            if quasi_random_uniform < self.sample_percentage {
                return Some(next);
            }
        }

        None
    }
}

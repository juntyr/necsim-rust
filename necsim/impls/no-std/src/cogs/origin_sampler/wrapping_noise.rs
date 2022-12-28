use core::{
    fmt,
    iter::{Iterator, Peekable},
};

use necsim_core::{
    cogs::{Habitat, MathsCore},
    landscape::{IndexedLocation, LandscapeExtent, LocationIterator},
    lineage::Lineage,
};

use crate::cogs::{
    habitat::wrapping_noise::WrappingNoiseHabitat,
    origin_sampler::{pre_sampler::OriginPreSampler, TrustedOriginSampler, UntrustedOriginSampler},
};

#[allow(clippy::module_name_repetitions)]
pub struct WrappingNoiseOriginSampler<'h, M: MathsCore, I: Iterator<Item = u64>> {
    pre_sampler: OriginPreSampler<I>,
    last_index: u64,
    location_iterator: Peekable<LocationIterator>,
    habitat: &'h WrappingNoiseHabitat<M>,
    sample: LandscapeExtent,
}

impl<'h, M: MathsCore, I: Iterator<Item = u64>> fmt::Debug
    for WrappingNoiseOriginSampler<'h, M, I>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(WrappingNoiseOriginSampler))
            .field("pre_sampler", &self.pre_sampler)
            .field("last_index", &self.last_index)
            .field("location_iterator", &self.location_iterator)
            .field("habitat", &self.habitat)
            .field("sample", &self.sample)
            .finish()
    }
}

impl<'h, M: MathsCore, I: Iterator<Item = u64>> WrappingNoiseOriginSampler<'h, M, I> {
    #[must_use]
    pub fn new(
        pre_sampler: OriginPreSampler<I>,
        habitat: &'h WrappingNoiseHabitat<M>,
        sample: LandscapeExtent,
    ) -> Self {
        Self {
            pre_sampler,
            last_index: 0_u64,
            location_iterator: sample.iter().peekable(),
            habitat,
            sample,
        }
    }
}

#[contract_trait]
impl<'h, M: MathsCore, I: Iterator<Item = u64>> UntrustedOriginSampler<'h, M>
    for WrappingNoiseOriginSampler<'h, M, I>
{
    type Habitat = WrappingNoiseHabitat<M>;
    type PreSampler = I;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn into_pre_sampler(self) -> OriginPreSampler<Self::PreSampler> {
        self.pre_sampler
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        {
            (f64::from(self.sample.width())
                * f64::from(self.sample.height())
                * self.pre_sampler.get_sample_proportion().get()) as u64
        }
    }
}

unsafe impl<'h, M: MathsCore, I: Iterator<Item = u64>> TrustedOriginSampler<'h, M>
    for WrappingNoiseOriginSampler<'h, M, I>
{
}

impl<'h, M: MathsCore, I: Iterator<Item = u64>> Iterator for WrappingNoiseOriginSampler<'h, M, I> {
    type Item = Lineage;

    fn next(&mut self) -> Option<Self::Item> {
        let next_index = self.pre_sampler.next()?;
        let mut index_difference = next_index - self.last_index;
        self.last_index = next_index + 1;

        for next_location in &mut self.location_iterator {
            if self.habitat.get_habitat_at_location(&next_location) > 0_u32 {
                if index_difference == 0 {
                    return Some(Lineage::new(
                        IndexedLocation::new(next_location, 0),
                        self.habitat,
                    ));
                }

                index_difference -= 1;
            }
        }

        None
    }
}

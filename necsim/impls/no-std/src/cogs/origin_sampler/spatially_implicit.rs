use core::{fmt, iter::Iterator};

use necsim_core::{cogs::OriginSampler, landscape::IndexedLocation};

use crate::cogs::{
    habitat::spatially_implicit::SpatiallyImplicitHabitat,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitOriginSampler<'h, I: Iterator<Item = u64>> {
    local_iterator: NonSpatialOriginSampler<'h, I>,
    habitat: &'h SpatiallyImplicitHabitat,
}

impl<'h, I: Iterator<Item = u64>> fmt::Debug for SpatiallyImplicitOriginSampler<'h, I> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SpatiallyImplicitOriginSampler")
            .field("local_iterator", &self.local_iterator)
            .field("habitat", &self.habitat)
            .finish()
    }
}

impl<'h, I: Iterator<Item = u64>> SpatiallyImplicitOriginSampler<'h, I> {
    #[must_use]
    pub fn new(pre_sampler: OriginPreSampler<I>, habitat: &'h SpatiallyImplicitHabitat) -> Self {
        Self {
            local_iterator: NonSpatialOriginSampler::new(pre_sampler, habitat.local()),
            habitat,
        }
    }
}

#[contract_trait]
impl<'h, I: Iterator<Item = u64>> OriginSampler<'h> for SpatiallyImplicitOriginSampler<'h, I> {
    type Habitat = SpatiallyImplicitHabitat;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.local_iterator.full_upper_bound_size_hint()
    }
}

impl<'h, I: Iterator<Item = u64>> Iterator for SpatiallyImplicitOriginSampler<'h, I> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        self.local_iterator.next()
    }
}

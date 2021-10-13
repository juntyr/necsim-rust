use core::{fmt, iter::Iterator};

use necsim_core::{
    cogs::{F64Core, OriginSampler},
    landscape::IndexedLocation,
};

use crate::cogs::{
    habitat::spatially_implicit::SpatiallyImplicitHabitat,
    origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitOriginSampler<'h, F: F64Core, I: Iterator<Item = u64>> {
    local_iterator: NonSpatialOriginSampler<'h, F, I>,
    habitat: &'h SpatiallyImplicitHabitat<F>,
}

impl<'h, F: F64Core, I: Iterator<Item = u64>> fmt::Debug
    for SpatiallyImplicitOriginSampler<'h, F, I>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.debug_struct(stringify!(SpatiallyImplicitOriginSampler))
            .field("local_iterator", &self.local_iterator)
            .field("habitat", &self.habitat)
            .finish()
    }
}

impl<'h, F: F64Core, I: Iterator<Item = u64>> SpatiallyImplicitOriginSampler<'h, F, I> {
    #[must_use]
    pub fn new(
        pre_sampler: OriginPreSampler<F, I>,
        habitat: &'h SpatiallyImplicitHabitat<F>,
    ) -> Self {
        Self {
            local_iterator: NonSpatialOriginSampler::new(pre_sampler, habitat.local()),
            habitat,
        }
    }
}

#[contract_trait]
impl<'h, F: F64Core, I: Iterator<Item = u64>> OriginSampler<'h, F>
    for SpatiallyImplicitOriginSampler<'h, F, I>
{
    type Habitat = SpatiallyImplicitHabitat<F>;

    fn habitat(&self) -> &'h Self::Habitat {
        self.habitat
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        self.local_iterator.full_upper_bound_size_hint()
    }
}

impl<'h, F: F64Core, I: Iterator<Item = u64>> Iterator
    for SpatiallyImplicitOriginSampler<'h, F, I>
{
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        self.local_iterator.next()
    }
}

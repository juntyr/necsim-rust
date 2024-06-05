use core::{fmt, iter::Iterator};

use necsim_core::{cogs::MathsCore, lineage::Lineage};

use crate::cogs::{
    habitat::almost_infinite::{
        downscaled::AlmostInfiniteDownscaledHabitat, AlmostInfiniteHabitat,
    },
    origin_sampler::{pre_sampler::OriginPreSampler, TrustedOriginSampler, UntrustedOriginSampler},
};

use super::{circle, rectangle};

#[allow(clippy::module_name_repetitions)]
pub enum AlmostInfiniteDownscaledOriginSampler<'h, M: MathsCore, I: Iterator<Item = u64>> {
    Circle(circle::SingletonDemesCircleOriginSampler<'h, M, AlmostInfiniteHabitat<M>, I>),
    Rectangle(rectangle::SingletonDemesRectangleOriginSampler<'h, M, AlmostInfiniteHabitat<M>, I>),
}

impl<'h, M: MathsCore, I: Iterator<Item = u64>> fmt::Debug
    for AlmostInfiniteDownscaledOriginSampler<'h, M, I>
{
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Circle(circle) => circle.fmt(fmt),
            Self::Rectangle(rectangle) => rectangle.fmt(fmt),
        }
    }
}

#[contract_trait]
impl<'h, M: MathsCore, I: Iterator<Item = u64>> UntrustedOriginSampler<'h, M>
    for AlmostInfiniteDownscaledOriginSampler<'h, M, I>
{
    type Habitat = AlmostInfiniteDownscaledHabitat<M>;
    type PreSampler = I;

    fn habitat(&self) -> &'h Self::Habitat {
        unimplemented!()
    }

    fn into_pre_sampler(self) -> OriginPreSampler<M, Self::PreSampler> {
        match self {
            Self::Circle(circle) => circle.into_pre_sampler(),
            Self::Rectangle(rectangle) => rectangle.into_pre_sampler(),
        }
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        match self {
            Self::Circle(circle) => circle.full_upper_bound_size_hint(),
            Self::Rectangle(rectangle) => rectangle.full_upper_bound_size_hint(),
        }
    }
}

unsafe impl<'h, M: MathsCore, I: Iterator<Item = u64>> TrustedOriginSampler<'h, M>
    for AlmostInfiniteDownscaledOriginSampler<'h, M, I>
{
}

impl<'h, M: MathsCore, I: Iterator<Item = u64>> Iterator
    for AlmostInfiniteDownscaledOriginSampler<'h, M, I>
{
    type Item = Lineage;

    fn next(&mut self) -> Option<Self::Item> {
        unimplemented!()
    }
}

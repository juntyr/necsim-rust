use core::{iter::Iterator, marker::PhantomData};

use necsim_core::{
    cogs::{Habitat, OriginSampler},
    intrinsics::ceil,
    landscape::IndexedLocation,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UniformPartitionOriginSampler<'h, H: Habitat, O: OriginSampler<'h, H>> {
    base_sampler: O,
    group_size: usize,
    _marker: PhantomData<&'h H>,
}

impl<'h, H: Habitat, O: OriginSampler<'h, H>> UniformPartitionOriginSampler<'h, H, O> {
    #[must_use]
    #[debug_requires(group_rank < group_size, "group_rank is in [0, group_size)")]
    pub fn new(mut base_sampler: O, group_rank: usize, group_size: usize) -> Self {
        let _ = base_sampler.advance_by(group_rank);

        Self {
            base_sampler,
            group_size,
            _marker: PhantomData::<&'h H>,
        }
    }
}

#[contract_trait]
impl<'h, H: Habitat, O: OriginSampler<'h, H>> OriginSampler<'h, H>
    for UniformPartitionOriginSampler<'h, H, O>
{
    fn habitat(&self) -> &'h H {
        self.base_sampler.habitat()
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        let upper_bound_size_hint = ceil(
            (self.base_sampler.full_upper_bound_size_hint() as f64) / (self.group_size as f64),
        ) as u64;

        upper_bound_size_hint
    }
}

impl<'h, H: Habitat, O: OriginSampler<'h, H>> Iterator for UniformPartitionOriginSampler<'h, H, O> {
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        let optional_next = self.base_sampler.next();

        let _ = self.base_sampler.advance_by(self.group_size - 1);

        optional_next
    }
}

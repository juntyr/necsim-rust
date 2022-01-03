use core::marker::PhantomData;

use necsim_core::{
    cogs::{Habitat, MathsCore},
    lineage::Lineage,
};

use crate::decomposition::Decomposition;

use super::{TrustedOriginSampler, UntrustedOriginSampler};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DecompositionOriginSampler<
    'd,
    M: MathsCore,
    O: UntrustedOriginSampler<'d, M>,
    D: Decomposition<M, O::Habitat>,
> {
    origin_sampler: O,
    decomposition: &'d D,
    _marker: PhantomData<M>,
}

impl<'d, M: MathsCore, O: UntrustedOriginSampler<'d, M>, D: Decomposition<M, O::Habitat>>
    DecompositionOriginSampler<'d, M, O, D>
{
    #[must_use]
    pub fn new(origin_sampler: O, decomposition: &'d D) -> Self {
        Self {
            origin_sampler,
            decomposition,
            _marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<'d, M: MathsCore, O: UntrustedOriginSampler<'d, M>, D: Decomposition<M, O::Habitat>>
    UntrustedOriginSampler<'d, M> for DecompositionOriginSampler<'d, M, O, D>
{
    type Habitat = O::Habitat;

    fn habitat(&self) -> &'d Self::Habitat {
        self.origin_sampler.habitat()
    }

    fn full_upper_bound_size_hint(&self) -> u64 {
        #[allow(
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss,
            clippy::cast_precision_loss
        )]
        {
            ((self.origin_sampler.full_upper_bound_size_hint() as f64)
                / f64::from(self.decomposition.get_subdomain().size().get())) as u64
        }
    }
}

unsafe impl<'d, M: MathsCore, O: TrustedOriginSampler<'d, M>, D: Decomposition<M, O::Habitat>>
    TrustedOriginSampler<'d, M> for DecompositionOriginSampler<'d, M, O, D>
{
}

impl<'d, M: MathsCore, O: UntrustedOriginSampler<'d, M>, D: Decomposition<M, O::Habitat>> Iterator
    for DecompositionOriginSampler<'d, M, O, D>
{
    type Item = Lineage;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::while_let_on_iterator)]
        while let Some(lineage) = self.origin_sampler.next() {
            // Forward any out-of-habitat or out-of-deme lineages
            if !self
                .origin_sampler
                .habitat()
                .contains(lineage.indexed_location.location())
                || lineage.indexed_location.index()
                    >= self
                        .origin_sampler
                        .habitat()
                        .get_habitat_at_location(lineage.indexed_location.location())
            {
                return Some(lineage);
            }

            if self.decomposition.map_location_to_subdomain_rank(
                lineage.indexed_location.location(),
                self.origin_sampler.habitat(),
            ) == self.decomposition.get_subdomain().rank()
            {
                return Some(lineage);
            }
        }

        None
    }
}

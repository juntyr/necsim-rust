use core::marker::PhantomData;

use necsim_core::{
    cogs::{F64Core, OriginSampler},
    landscape::IndexedLocation,
};

use crate::decomposition::Decomposition;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct DecompositionOriginSampler<
    'd,
    F: F64Core,
    O: OriginSampler<'d, F>,
    D: Decomposition<F, O::Habitat>,
> {
    origin_sampler: O,
    decomposition: &'d D,
    _marker: PhantomData<F>,
}

impl<'d, F: F64Core, O: OriginSampler<'d, F>, D: Decomposition<F, O::Habitat>>
    DecompositionOriginSampler<'d, F, O, D>
{
    #[must_use]
    pub fn new(origin_sampler: O, decomposition: &'d D) -> Self {
        Self {
            origin_sampler,
            decomposition,
            _marker: PhantomData::<F>,
        }
    }
}

#[contract_trait]
impl<'d, F: F64Core, O: OriginSampler<'d, F>, D: Decomposition<F, O::Habitat>> OriginSampler<'d, F>
    for DecompositionOriginSampler<'d, F, O, D>
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
                / f64::from(self.decomposition.get_number_of_subdomains().get())) as u64
        }
    }
}

impl<'d, F: F64Core, O: OriginSampler<'d, F>, D: Decomposition<F, O::Habitat>> Iterator
    for DecompositionOriginSampler<'d, F, O, D>
{
    type Item = IndexedLocation;

    fn next(&mut self) -> Option<Self::Item> {
        #[allow(clippy::while_let_on_iterator)]
        while let Some(indexed_location) = self.origin_sampler.next() {
            if self.decomposition.map_location_to_subdomain_rank(
                indexed_location.location(),
                self.origin_sampler.habitat(),
            ) == self.decomposition.get_subdomain_rank()
            {
                return Some(indexed_location);
            }
        }

        None
    }
}

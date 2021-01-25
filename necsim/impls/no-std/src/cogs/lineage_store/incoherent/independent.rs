use core::{marker::PhantomData, ops::Index};

use necsim_core::{
    cogs::{Habitat, IncoherentLineageStore, LineageStore},
    landscape::IndexedLocation,
    lineage::Lineage,
};

use crate::cogs::active_lineage_sampler::independent::lineage_reference::IndependentLineageReference;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentLineageStore<H: Habitat> {
    marker: PhantomData<H>,
}

impl<H: Habitat> Index<IndependentLineageReference> for IndependentLineageStore<H> {
    type Output = Lineage;

    fn index(&self, _reference: IndependentLineageReference) -> &Self::Output {
        unreachable!("This should implement a subtrait instead")
    }
}

impl<H: Habitat> Default for IndependentLineageStore<H> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<H: Habitat> LineageStore<H, IndependentLineageReference> for IndependentLineageStore<H> {
    type LineageReferenceIterator<'a> = core::iter::Empty<IndependentLineageReference>;

    #[must_use]
    fn get_number_total_lineages(&self) -> usize {
        0_usize
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        core::iter::empty()
    }

    #[must_use]
    fn get(&self, _reference: IndependentLineageReference) -> Option<&Lineage> {
        None
    }

    fn update_lineage_time_of_last_event(
        &mut self,
        _reference: IndependentLineageReference,
        _event_time: f64,
    ) {
        // no-op
    }
}

#[contract_trait]
impl<H: Habitat> IncoherentLineageStore<H, IndependentLineageReference>
    for IndependentLineageStore<H>
{
    fn insert_lineage_to_indexed_location(
        &mut self,
        _reference: IndependentLineageReference,
        _indexed_location: IndexedLocation,
        _habitat: &H,
    ) {
        // no-op
    }

    #[must_use]
    fn extract_lineage_from_its_location(
        &mut self,
        _reference: IndependentLineageReference,
        _habitat: &H,
    ) -> IndexedLocation {
        unreachable!("This should implement a subtrait instead")
    }
}

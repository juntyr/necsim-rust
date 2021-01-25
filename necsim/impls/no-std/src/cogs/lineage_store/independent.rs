use core::marker::PhantomData;

use alloc::vec::Vec;

use necsim_core::{
    cogs::{Habitat, LineageStore},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentLineageStore<H: Habitat> {
    marker: PhantomData<H>,
}

impl<H: Habitat> Default for IndependentLineageStore<H> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<H: Habitat> LineageStore<H, GlobalLineageReference> for IndependentLineageStore<H> {
    type LineageReferenceIterator<'a> = core::iter::Empty<GlobalLineageReference>;

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
    fn get(&self, _reference: GlobalLineageReference) -> Option<&Lineage> {
        None
    }

    #[must_use]
    fn into_lineages(self) -> Vec<Lineage> {
        Vec::new()
    }
}

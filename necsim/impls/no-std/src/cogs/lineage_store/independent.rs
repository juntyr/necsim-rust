use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, Habitat, LineageStore, OriginSampler},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
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
impl<H: Habitat> Backup for IndependentLineageStore<H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<H: Habitat> LineageStore<H, GlobalLineageReference> for IndependentLineageStore<H> {
    type LineageReferenceIterator<'a> = core::iter::Empty<GlobalLineageReference>;

    fn from_origin_sampler<'h, O: OriginSampler<'h, Habitat = H>>(_origin_sampler: O) -> Self
    where
        H: 'h,
    {
        Self::default()
    }

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
}

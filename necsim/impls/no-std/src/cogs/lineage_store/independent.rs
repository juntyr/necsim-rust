use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, Habitat, LineageStore, OriginSampler, F64Core},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
pub struct IndependentLineageStore<F: F64Core, H: Habitat<F>> {
    marker: PhantomData<(F, H)>,
}

impl<F: F64Core, H: Habitat<F>> Default for IndependentLineageStore<F, H> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> Backup for IndependentLineageStore<F, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<H>,
        }
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> LineageStore<F, H, GlobalLineageReference> for IndependentLineageStore<F, H> {
    type LineageReferenceIterator<'a> = core::iter::Empty<GlobalLineageReference>;

    fn from_origin_sampler<'h, O: OriginSampler<'h, F, Habitat = H>>(_origin_sampler: O) -> Self
    where
        H: 'h,
    {
        Self::default()
    }

    #[must_use]
    #[must_use]
    fn iter_local_lineage_references(&self) -> Self::LineageReferenceIterator<'_> {
        core::iter::empty()
    }

    #[must_use]
    fn get_lineage_for_local_reference(
        &self,
        _reference: GlobalLineageReference,
    ) -> Option<&Lineage> {
        None
    }
}

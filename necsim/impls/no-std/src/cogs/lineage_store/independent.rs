use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, Habitat, LineageStore, MathsCore, OriginSampler},
    lineage::{GlobalLineageReference, Lineage},
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
pub struct IndependentLineageStore<M: MathsCore, H: Habitat<M>> {
    marker: PhantomData<(M, H)>,
}

impl<M: MathsCore, H: Habitat<M>> Default for IndependentLineageStore<M, H> {
    fn default() -> Self {
        Self {
            marker: PhantomData::<(M, H)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> Backup for IndependentLineageStore<M, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            marker: PhantomData::<(M, H)>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> LineageStore<M, H, GlobalLineageReference>
    for IndependentLineageStore<M, H>
{
    type LineageReferenceIterator<'a>
    where
        H: 'a,
    = impl Iterator<Item = GlobalLineageReference>;

    fn from_origin_sampler<'h, O: OriginSampler<'h, M, Habitat = H>>(_origin_sampler: O) -> Self
    where
        H: 'h,
    {
        Self::default()
    }

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

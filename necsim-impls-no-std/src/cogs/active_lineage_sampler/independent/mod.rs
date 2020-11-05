use core::marker::PhantomData;

use necsim_core::cogs::{DispersalSampler, Habitat, IncoherentLineageStore, LineageReference};

mod sampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: necsim_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(S: necsim_cuda::common::RustToCuda))]
pub struct IndependentActiveLineageSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
> {
    // TODO: This reference needs to somehow be initialised by the thread index in CUDA whilst allowing for generalisation
    active_lineage_reference: Option<R>,
    marker: PhantomData<(H, D, S)>,
}

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > IndependentActiveLineageSampler<H, D, R, S>
{
    #[must_use]
    pub fn new(active_lineage_reference: R, lineage_store: &S) -> Self {
        Self {
            active_lineage_reference: lineage_store
                .get(active_lineage_reference.clone())
                .map(|_| active_lineage_reference),
            marker: PhantomData::<(H, D, S)>,
        }
    }
}

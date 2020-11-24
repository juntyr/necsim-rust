use core::marker::PhantomData;

use necsim_core::cogs::{
    DispersalSampler, HabitatToU64Injection, IncoherentLineageStore, LineageReference, PrimeableRng,
};

mod sampler;
mod singular;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentActiveLineageSampler<
    H: HabitatToU64Injection,
    G: PrimeableRng<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
> {
    active_lineage_reference: Option<R>,
    marker: PhantomData<(H, G, D, S)>,
}

impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > IndependentActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    pub fn new(active_lineage_reference: R, lineage_store: &S) -> Self {
        Self {
            active_lineage_reference: lineage_store
                .get(active_lineage_reference.clone())
                .map(|_| active_lineage_reference),
            marker: PhantomData::<(H, G, D, S)>,
        }
    }
}

impl<
        H: HabitatToU64Injection,
        G: PrimeableRng<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
    > Default for IndependentActiveLineageSampler<H, G, D, R, S>
{
    #[must_use]
    fn default() -> Self {
        Self {
            active_lineage_reference: None,
            marker: PhantomData::<(H, G, D, S)>,
        }
    }
}

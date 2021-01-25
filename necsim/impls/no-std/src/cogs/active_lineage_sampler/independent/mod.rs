use core::marker::PhantomData;

use necsim_core::{
    cogs::{DispersalSampler, EmigrationExit, Habitat, PrimeableRng, SpeciationProbability},
    lineage::Lineage,
};

use crate::cogs::lineage_store::incoherent::independent::IndependentLineageStore;

mod sampler;
mod singular;

pub mod event_time_sampler;
pub mod lineage_reference;

use event_time_sampler::EventTimeSampler;
use lineage_reference::IndependentLineageReference;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentActiveLineageSampler<
    H: Habitat,
    G: PrimeableRng<H>,
    N: SpeciationProbability<H>,
    T: EventTimeSampler<H, G>,
    D: DispersalSampler<H, G>,
    X: EmigrationExit<H, G, N, D, IndependentLineageReference, IndependentLineageStore<H>>,
> {
    active_lineage: Option<Lineage>,
    event_time_sampler: T,
    marker: PhantomData<(H, G, N, D, X)>,
}

impl<
        H: Habitat,
        G: PrimeableRng<H>,
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        X: EmigrationExit<H, G, N, D, IndependentLineageReference, IndependentLineageStore<H>>,
    > IndependentActiveLineageSampler<H, G, N, T, D, X>
{
    #[must_use]
    pub fn empty(event_time_sampler: T) -> Self {
        Self {
            active_lineage: None,
            event_time_sampler,
            marker: PhantomData::<(H, G, N, D, X)>,
        }
    }
}

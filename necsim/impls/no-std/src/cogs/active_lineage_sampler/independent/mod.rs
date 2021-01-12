use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        DispersalSampler, EmigrationExit, Habitat, IncoherentLineageStore, LineageReference,
        PrimeableRng, SpeciationProbability,
    },
    landscape::IndexedLocation,
};

mod sampler;
mod singular;

pub mod event_time_sampler;

use event_time_sampler::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentActiveLineageSampler<
    H: Habitat,
    G: PrimeableRng<H>,
    N: SpeciationProbability<H>,
    T: EventTimeSampler<H, G>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
> {
    active_lineage_reference: Option<R>,
    lineage_indexed_location: Option<IndexedLocation>,
    lineage_time_of_last_event: f64,
    event_time_sampler: T,
    marker: PhantomData<(H, G, N, D, S, X)>,
}

impl<
        H: Habitat,
        G: PrimeableRng<H>,
        N: SpeciationProbability<H>,
        T: EventTimeSampler<H, G>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: IncoherentLineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
    > IndependentActiveLineageSampler<H, G, N, T, D, R, S, X>
{
    #[must_use]
    pub fn new_from(
        event_time_sampler: T,
        active_lineage_reference: R,
        lineage_store: &mut S,
    ) -> Self {
        #[allow(clippy::option_if_let_else)]
        if let Some(lineage) = lineage_store.get(active_lineage_reference.clone()) {
            if lineage.is_active() {
                return Self {
                    active_lineage_reference: Some(active_lineage_reference.clone()),
                    lineage_time_of_last_event: lineage.time_of_last_event(),
                    lineage_indexed_location: Some(
                        lineage_store.extract_lineage_from_its_location(active_lineage_reference),
                    ),
                    event_time_sampler,
                    marker: PhantomData::<(H, G, N, D, S, X)>,
                };
            }
        }

        Self::empty(event_time_sampler)
    }

    #[must_use]
    pub fn empty(event_time_sampler: T) -> Self {
        Self {
            active_lineage_reference: None,
            lineage_indexed_location: None,
            lineage_time_of_last_event: 0.0_f64,
            event_time_sampler,
            marker: PhantomData::<(H, G, N, D, S, X)>,
        }
    }
}

use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, DispersalSampler, EmigrationExit, Habitat, PrimeableRng, SpeciationProbability,
        TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
};
use necsim_core_bond::PositiveF64;

use crate::cogs::lineage_store::independent::IndependentLineageStore;

mod sampler;
mod singular;

pub mod event_time_sampler;

use event_time_sampler::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(J: rust_cuda::common::RustToCuda))]
pub struct IndependentActiveLineageSampler<
    H: Habitat,
    G: PrimeableRng,
    X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
    D: DispersalSampler<H, G>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
    J: EventTimeSampler<H, G, T>,
> {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    active_lineage: Option<Lineage>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    next_event_time: Option<PositiveF64>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    event_time_sampler: J,
    marker: PhantomData<(H, G, X, D, T, N)>,
}

impl<
        H: Habitat,
        G: PrimeableRng,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        J: EventTimeSampler<H, G, T>,
    > IndependentActiveLineageSampler<H, G, X, D, T, N, J>
{
    #[must_use]
    pub fn empty(event_time_sampler: J) -> Self {
        Self {
            active_lineage: None,
            next_event_time: None,
            event_time_sampler,
            marker: PhantomData::<(H, G, X, D, T, N)>,
        }
    }
}

#[contract_trait]
impl<
        H: Habitat,
        G: PrimeableRng,
        X: EmigrationExit<H, G, GlobalLineageReference, IndependentLineageStore<H>>,
        D: DispersalSampler<H, G>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        J: EventTimeSampler<H, G, T>,
    > Backup for IndependentActiveLineageSampler<H, G, X, D, T, N, J>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage: self.active_lineage.clone(),
            next_event_time: self.next_event_time,
            event_time_sampler: self.event_time_sampler.clone(),
            marker: PhantomData::<(H, G, X, D, T, N)>,
        }
    }
}

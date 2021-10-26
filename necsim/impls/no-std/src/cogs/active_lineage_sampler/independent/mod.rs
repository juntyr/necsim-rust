use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, DispersalSampler, EmigrationExit, Habitat, MathsCore, PrimeableRng,
        SpeciationProbability, TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
};
use necsim_core_bond::NonNegativeF64;

use crate::cogs::lineage_store::independent::IndependentLineageStore;

mod sampler;
mod singular;

pub mod event_time_sampler;

use event_time_sampler::EventTimeSampler;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(X: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(D: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(T: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(N: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(J: rust_cuda::common::RustToCuda))]
pub struct IndependentActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: PrimeableRng<M>,
    X: EmigrationExit<M, H, G, GlobalLineageReference, IndependentLineageStore<M, H>>,
    D: DispersalSampler<M, H, G>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    J: EventTimeSampler<M, H, G, T>,
> {
    #[cfg_attr(feature = "cuda", r2cEmbed(
        Option<rust_cuda::utils::device_copy::SafeDeviceCopyWrapper<Lineage>>
    ))]
    active_lineage: Option<Lineage>,
    last_event_time: NonNegativeF64,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    event_time_sampler: J,
    marker: PhantomData<(M, H, G, X, D, T, N)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: PrimeableRng<M>,
        X: EmigrationExit<M, H, G, GlobalLineageReference, IndependentLineageStore<M, H>>,
        D: DispersalSampler<M, H, G>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        J: EventTimeSampler<M, H, G, T>,
    > IndependentActiveLineageSampler<M, H, G, X, D, T, N, J>
{
    #[must_use]
    pub fn empty(event_time_sampler: J) -> Self {
        Self {
            active_lineage: None,
            last_event_time: NonNegativeF64::zero(),
            event_time_sampler,
            marker: PhantomData::<(M, H, G, X, D, T, N)>,
        }
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: PrimeableRng<M>,
        X: EmigrationExit<M, H, G, GlobalLineageReference, IndependentLineageStore<M, H>>,
        D: DispersalSampler<M, H, G>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        J: EventTimeSampler<M, H, G, T>,
    > Backup for IndependentActiveLineageSampler<M, H, G, X, D, T, N, J>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage: self.active_lineage.clone(),
            last_event_time: self.last_event_time,
            event_time_sampler: self.event_time_sampler.clone(),
            marker: PhantomData::<(M, H, G, X, D, T, N)>,
        }
    }
}

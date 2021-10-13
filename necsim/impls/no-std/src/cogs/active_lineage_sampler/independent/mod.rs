use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        Backup, DispersalSampler, EmigrationExit, F64Core, Habitat, PrimeableRng,
        SpeciationProbability, TurnoverRate,
    },
    lineage::{GlobalLineageReference, Lineage},
};

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
    F: F64Core,
    H: Habitat<F>,
    G: PrimeableRng<F>,
    X: EmigrationExit<F, H, G, GlobalLineageReference, IndependentLineageStore<F, H>>,
    D: DispersalSampler<F, H, G>,
    T: TurnoverRate<F, H>,
    N: SpeciationProbability<F, H>,
    J: EventTimeSampler<F, H, G, T>,
> {
    #[cfg_attr(feature = "cuda", r2cEmbed(
        Option<rust_cuda::utils::device_copy::SafeDeviceCopyWrapper<Lineage>>
    ))]
    active_lineage: Option<Lineage>,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    event_time_sampler: J,
    marker: PhantomData<(F, H, G, X, D, T, N)>,
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: PrimeableRng<F>,
        X: EmigrationExit<F, H, G, GlobalLineageReference, IndependentLineageStore<F, H>>,
        D: DispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        J: EventTimeSampler<F, H, G, T>,
    > IndependentActiveLineageSampler<F, H, G, X, D, T, N, J>
{
    #[must_use]
    pub fn empty(event_time_sampler: J) -> Self {
        Self {
            active_lineage: None,
            event_time_sampler,
            marker: PhantomData::<(F, H, G, X, D, T, N)>,
        }
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: PrimeableRng<F>,
        X: EmigrationExit<F, H, G, GlobalLineageReference, IndependentLineageStore<F, H>>,
        D: DispersalSampler<F, H, G>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        J: EventTimeSampler<F, H, G, T>,
    > Backup for IndependentActiveLineageSampler<F, H, G, X, D, T, N, J>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            active_lineage: self.active_lineage.clone(),
            event_time_sampler: self.event_time_sampler.clone(),
            marker: PhantomData::<(F, H, G, X, D, T, N)>,
        }
    }
}

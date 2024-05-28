use necsim_core::{
    cogs::{DispersalSampler, EmigrationExit, MathsCore, PrimeableRng},
    lineage::Lineage,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::event_time_sampler::EventTimeSampler, singular::SingularActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::in_memory::packed_separable_alias::InMemoryPackedSeparableAliasDispersalSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::TrustedOriginSampler,
};

use rustcoalescence_scenarios::Scenario;

use rust_cuda::lend::RustToCuda;

use crate::CudaError;

pub mod fixup;
pub mod genesis;
pub mod resume;

#[allow(clippy::module_name_repetitions)]
pub trait CudaLineageStoreSampleInitialiser<
    M: MathsCore,
    G: PrimeableRng<M> + RustToCuda + Sync,
    O: Scenario<M, G>,
    Error: From<CudaError>,
> where
    O::Habitat: RustToCuda + Sync,
    O::DispersalSampler<InMemoryPackedSeparableAliasDispersalSampler<M, O::Habitat, G>>:
        RustToCuda + Sync,
    O::TurnoverRate: RustToCuda + Sync,
    O::SpeciationProbability: RustToCuda + Sync,
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G> + RustToCuda + Sync;
    type ActiveLineageSampler<
        X: EmigrationExit<
            M,
            O::Habitat,
            G,
            IndependentLineageStore<M, O::Habitat>,
        > + RustToCuda + Sync,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda + Sync,
    >: SingularActiveLineageSampler<
        M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>,
        X, Self::DispersalSampler, IndependentCoalescenceSampler<M, O::Habitat>, O::TurnoverRate,
        O::SpeciationProbability, IndependentEventSampler<
            M, O::Habitat, G, X, Self::DispersalSampler, O::TurnoverRate, O::SpeciationProbability
        >, NeverImmigrationEntry,
    > + RustToCuda + Sync;

    #[allow(clippy::type_complexity)]
    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda + Sync,
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>>
            + RustToCuda
            + Sync,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemoryPackedSeparableAliasDispersalSampler<M, O::Habitat, G>,
        >,
        event_time_sampler: J,
    ) -> Result<
        (
            IndependentLineageStore<M, O::Habitat>,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<X, J>,
            Vec<Lineage>,
            Vec<Lineage>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}

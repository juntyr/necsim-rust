use necsim_core::{
    cogs::{DispersalSampler, EmigrationExit, MathsCore, PrimeableRng},
    lineage::Lineage,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::event_time_sampler::EventTimeSampler, singular::SingularActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::TrustedOriginSampler,
};

use rustcoalescence_scenarios::Scenario;

use rust_cuda::common::RustToCuda;

use crate::CudaError;

pub mod fixup;
pub mod genesis;
pub mod resume;

#[allow(clippy::module_name_repetitions)]
pub trait CudaLineageStoreSampleInitialiser<
    M: MathsCore,
    G: PrimeableRng<M> + RustToCuda,
    O: Scenario<M, G>,
    Error: From<CudaError>,
> where
    O::Habitat: RustToCuda,
    O::DispersalSampler<InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>>: RustToCuda,
    O::TurnoverRate: RustToCuda,
    O::SpeciationProbability: RustToCuda,
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G> + RustToCuda;
    type ActiveLineageSampler<
        X: EmigrationExit<
            M,
            O::Habitat,
            G,
            IndependentLineageStore<M, O::Habitat>,
        > + RustToCuda,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
    >: SingularActiveLineageSampler<
        M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>,
        X, Self::DispersalSampler, IndependentCoalescenceSampler<M, O::Habitat>, O::TurnoverRate,
        O::SpeciationProbability, IndependentEventSampler<
            M, O::Habitat, G, X, Self::DispersalSampler, O::TurnoverRate, O::SpeciationProbability
        >, NeverImmigrationEntry,
    > + RustToCuda;

    #[allow(clippy::type_complexity)]
    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate> + RustToCuda,
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>> + RustToCuda,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemoryPackedAliasDispersalSampler<M, O::Habitat, G>,
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

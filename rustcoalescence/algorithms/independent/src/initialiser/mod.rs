use necsim_core::{
    cogs::{
        rng::{Event, IndexUsize, UniformClosedOpenUnit},
        DispersalSampler, DistributionSampler, EmigrationExit, MathsCore, PrimeableRng, Rng,
    },
    lineage::Lineage,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::{
        independent::event_time_sampler::EventTimeSampler, singular::SingularActiveLineageSampler,
    },
    coalescence_sampler::independent::IndependentCoalescenceSampler,
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    event_sampler::independent::IndependentEventSampler,
    immigration_entry::never::NeverImmigrationEntry,
    lineage_store::independent::IndependentLineageStore,
    origin_sampler::TrustedOriginSampler,
};

use rustcoalescence_scenarios::Scenario;

pub mod fixup;
pub mod genesis;
pub mod resume;

#[allow(clippy::module_name_repetitions)]
pub trait IndependentLineageStoreSampleInitialiser<
    M: MathsCore,
    G: Rng<M, Generator: PrimeableRng>,
    O: Scenario<M, G>,
    Error,
> where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>
        + DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Event>,
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<X: EmigrationExit<
        M,
        O::Habitat,
        G,
        IndependentLineageStore<M, O::Habitat>,
    >, J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate>>: SingularActiveLineageSampler<
        M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>,
        X, Self::DispersalSampler, IndependentCoalescenceSampler<M, O::Habitat>, O::TurnoverRate,
        O::SpeciationProbability, IndependentEventSampler<
            M, O::Habitat, G, X, Self::DispersalSampler, O::TurnoverRate, O::SpeciationProbability
        >, NeverImmigrationEntry,
    >;

    #[allow(clippy::type_complexity)]
    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        J: EventTimeSampler<M, O::Habitat, G, O::TurnoverRate>,
        X: EmigrationExit<M, O::Habitat, G, IndependentLineageStore<M, O::Habitat>>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
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

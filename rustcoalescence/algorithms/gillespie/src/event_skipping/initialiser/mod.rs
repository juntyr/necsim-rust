use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize, UniformClosedOpenUnit},
        ActiveLineageSampler, EmigrationExit, GloballyCoherentLineageStore, ImmigrationEntry,
        MathsCore, Rng, Samples, SeparableDispersalSampler,
    },
    reporter::Reporter,
};

use necsim_impls_no_std::cogs::{
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
    event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
    origin_sampler::TrustedOriginSampler,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

pub mod fixup;
pub mod genesis;
pub mod resume;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::trait_duplication_in_bounds)]
pub trait EventSkippingLineageStoreSampleInitialiser<
    M: MathsCore,
    G: Rng<M> + Samples<M, IndexUsize> + Samples<M, Bernoulli> + Samples<M, UniformClosedOpenUnit>,
    O: Scenario<M, G>,
    Error,
> where
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    type DispersalSampler: SeparableDispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        S: GloballyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        I: ImmigrationEntry<M>,
    >: ActiveLineageSampler<
        M,
        O::Habitat,
        G,
        S,
        X,
        Self::DispersalSampler,
        ConditionalCoalescenceSampler<M, O::Habitat, S>,
        O::TurnoverRate,
        O::SpeciationProbability,
        ConditionalGillespieEventSampler<
            M,
            O::Habitat,
            G,
            S,
            X,
            Self::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    #[allow(clippy::type_complexity)]
    fn init<
        'h,
        'p,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<'p, Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
        >,
        coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, S>,
        turnover_rate: &O::TurnoverRate,
        speciation_probability: &O::SpeciationProbability,
        local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            ConditionalGillespieEventSampler<
                M,
                O::Habitat,
                G,
                S,
                X,
                Self::DispersalSampler,
                O::TurnoverRate,
                O::SpeciationProbability,
            >,
            Self::ActiveLineageSampler<S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}

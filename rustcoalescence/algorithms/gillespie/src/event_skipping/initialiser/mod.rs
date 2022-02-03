use necsim_core::{
    cogs::{
        ActiveLineageSampler, EmigrationExit, GloballyCoherentLineageStore, ImmigrationEntry,
        LineageReference, MathsCore, RngCore, SeparableDispersalSampler,
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

#[allow(clippy::type_complexity)]
#[allow(clippy::module_name_repetitions)]
pub trait EventSkippingLineageStoreSampleInitialiser<
    M: MathsCore,
    G: RngCore<M>,
    O: Scenario<M, G>,
    Error,
> where
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    type DispersalSampler: SeparableDispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
    >: ActiveLineageSampler<
        M,
        O::Habitat,
        G,
        R,
        S,
        X,
        Self::DispersalSampler,
        ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
        O::TurnoverRate,
        O::SpeciationProbability,
        ConditionalGillespieEventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        R: LineageReference<M, O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<
            InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>,
        >,
        coalescence_sampler: &ConditionalCoalescenceSampler<M, O::Habitat, R, S>,
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
                R,
                S,
                X,
                Self::DispersalSampler,
                O::TurnoverRate,
                O::SpeciationProbability,
            >,
            Self::ActiveLineageSampler<R, S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}

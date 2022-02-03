use necsim_core::{
    cogs::{
        EmigrationExit, GloballyCoherentLineageStore, ImmigrationEntry, LineageReference,
        MathsCore, RngCore, SeparableDispersalSampler,
    },
    reporter::Reporter,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::alias::location::LocationAliasActiveLineageSampler,
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
    event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
    origin_sampler::TrustedOriginSampler,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use super::EventSkippingLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct GenesisInitialiser;

#[allow(clippy::type_complexity)]
impl<M: MathsCore, G: RngCore<M>, O: Scenario<M, G>>
    EventSkippingLineageStoreSampleInitialiser<M, G, O, !> for GenesisInitialiser
where
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
{
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: GloballyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
    > = LocationAliasActiveLineageSampler<
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
    type DispersalSampler =
        O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>;

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
        _local_partition: &mut P,
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
        !,
    >
    where
        O::Habitat: 'h,
    {
        let event_sampler = ConditionalGillespieEventSampler::default();

        let (lineage_store, active_lineage_sampler) =
            LocationAliasActiveLineageSampler::init_with_store(
                origin_sampler,
                &dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                &event_sampler,
            );

        Ok((
            lineage_store,
            dispersal_sampler,
            event_sampler,
            active_lineage_sampler,
        ))
    }
}

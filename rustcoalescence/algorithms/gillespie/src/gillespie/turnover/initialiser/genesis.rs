use necsim_core::{
    cogs::{
        CoalescenceSampler, EmigrationExit, EventSampler, ImmigrationEntry, LineageReference,
        LocallyCoherentLineageStore, MathsCore, RngCore,
    },
    reporter::Reporter,
};

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::alias::individual::IndividualAliasActiveLineageSampler,
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    origin_sampler::TrustedOriginSampler,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

use super::GillespieLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct GenesisInitialiser;

#[allow(clippy::type_complexity)]
impl<M: MathsCore, G: RngCore<M>, O: Scenario<M, G>>
    GillespieLineageStoreSampleInitialiser<M, G, O, !> for GenesisInitialiser
{
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        C: CoalescenceSampler<M, O::Habitat, R, S>,
        E: EventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            C,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I: ImmigrationEntry<M>,
    > = IndividualAliasActiveLineageSampler<
        M,
        O::Habitat,
        G,
        R,
        S,
        X,
        Self::DispersalSampler,
        C,
        O::TurnoverRate,
        O::SpeciationProbability,
        E,
        I,
    >;
    type DispersalSampler = O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        C: CoalescenceSampler<M, O::Habitat, R, S>,
        E: EventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            C,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
        turnover_rate: &O::TurnoverRate,
        _local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<R, S, X, C, E, I>,
        ),
        !,
    >
    where
        O::Habitat: 'h,
    {
        let (lineage_store, active_lineage_sampler) =
            IndividualAliasActiveLineageSampler::init_with_store(origin_sampler, turnover_rate);

        Ok((lineage_store, dispersal_sampler, active_lineage_sampler))
    }
}

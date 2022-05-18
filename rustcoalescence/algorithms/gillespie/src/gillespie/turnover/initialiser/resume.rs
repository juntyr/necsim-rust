use necsim_core::{
    cogs::{
        rng::{Event, Exponential, IndexU128, IndexU64, IndexUsize},
        CoalescenceSampler, DistributionSampler, EmigrationExit, EventSampler, ImmigrationEntry,
        LocallyCoherentLineageStore, MathsCore, Rng,
    },
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::alias::individual::IndividualAliasActiveLineageSampler,
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    origin_sampler::{resuming::ResumingOriginSampler, TrustedOriginSampler},
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::result::ResumeError;
use rustcoalescence_scenarios::Scenario;

use super::GillespieLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct ResumeInitialiser<L: ExactSizeIterator<Item = Lineage>> {
    pub lineages: L,
    pub resume_after: Option<NonNegativeF64>,
}

impl<L: ExactSizeIterator<Item = Lineage>, M: MathsCore, G: Rng<M>, O: Scenario<M, G>>
    GillespieLineageStoreSampleInitialiser<M, G, O, ResumeError<!>> for ResumeInitialiser<L>
where
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Event>
        + DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, IndexU128>
        + DistributionSampler<M, G::Generator, G::Sampler, Exponential>,
{
    type ActiveLineageSampler<
        S: LocallyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        C: CoalescenceSampler<M, O::Habitat, S>,
        E: EventSampler<
            M,
            O::Habitat,
            G,
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
        'p,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        C: CoalescenceSampler<M, O::Habitat, S>,
        E: EventSampler<
            M,
            O::Habitat,
            G,
            S,
            X,
            Self::DispersalSampler,
            C,
            O::TurnoverRate,
            O::SpeciationProbability,
        >,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<'p, Q>,
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
            Self::ActiveLineageSampler<S, X, C, E, I>,
        ),
        ResumeError<!>,
    >
    where
        O::Habitat: 'h,
    {
        let habitat = origin_sampler.habitat();
        let pre_sampler = origin_sampler.into_pre_sampler();

        let (lineage_store, active_lineage_sampler, exceptional_lineages) =
            IndividualAliasActiveLineageSampler::resume_with_store(
                ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                turnover_rate,
                self.resume_after.unwrap_or(NonNegativeF64::zero()),
            );

        if !exceptional_lineages.is_empty() {
            return Err(ResumeError::Sample(exceptional_lineages));
        }

        Ok((lineage_store, dispersal_sampler, active_lineage_sampler))
    }
}

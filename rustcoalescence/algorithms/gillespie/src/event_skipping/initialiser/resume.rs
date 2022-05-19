use necsim_core::{
    cogs::{
        distribution::{
            Bernoulli, Exponential, IndexU128, IndexU64, IndexUsize, UniformClosedOpenUnit,
        },
        DistributionSampler, EmigrationExit, GloballyCoherentLineageStore, ImmigrationEntry,
        MathsCore, Rng, SeparableDispersalSampler,
    },
    lineage::Lineage,
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::cogs::{
    active_lineage_sampler::alias::location::LocationAliasActiveLineageSampler,
    coalescence_sampler::conditional::ConditionalCoalescenceSampler,
    dispersal_sampler::in_memory::separable_alias::InMemorySeparableAliasDispersalSampler,
    event_sampler::gillespie::conditional::ConditionalGillespieEventSampler,
    origin_sampler::{resuming::ResumingOriginSampler, TrustedOriginSampler},
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_algorithms::result::ResumeError;
use rustcoalescence_scenarios::Scenario;

use super::EventSkippingLineageStoreSampleInitialiser;

#[allow(clippy::module_name_repetitions)]
pub struct ResumeInitialiser<L: ExactSizeIterator<Item = Lineage>> {
    pub lineages: L,
    pub resume_after: Option<NonNegativeF64>,
}

#[allow(clippy::type_complexity)]
impl<L: ExactSizeIterator<Item = Lineage>, M: MathsCore, G: Rng<M>, O: Scenario<M, G>>
    EventSkippingLineageStoreSampleInitialiser<M, G, O, ResumeError<!>> for ResumeInitialiser<L>
where
    O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>:
        SeparableDispersalSampler<M, O::Habitat, G>,
    G::Sampler: DistributionSampler<M, G::Generator, G::Sampler, IndexUsize>
        + DistributionSampler<M, G::Generator, G::Sampler, Bernoulli>
        + DistributionSampler<M, G::Generator, G::Sampler, UniformClosedOpenUnit>
        + DistributionSampler<M, G::Generator, G::Sampler, Exponential>
        + DistributionSampler<M, G::Generator, G::Sampler, IndexU64>
        + DistributionSampler<M, G::Generator, G::Sampler, IndexU128>,
{
    type ActiveLineageSampler<
        S: GloballyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        I: ImmigrationEntry<M>,
    > = LocationAliasActiveLineageSampler<
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
    type DispersalSampler =
        O::DispersalSampler<InMemorySeparableAliasDispersalSampler<M, O::Habitat, G>>;

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
        _local_partition: &mut P,
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
        ResumeError<!>,
    >
    where
        O::Habitat: 'h,
    {
        let habitat = origin_sampler.habitat();
        let pre_sampler = origin_sampler.into_pre_sampler();

        let event_sampler = ConditionalGillespieEventSampler::default();

        let (lineage_store, active_lineage_sampler, exceptional_lineages) =
            LocationAliasActiveLineageSampler::resume_with_store(
                ResumingOriginSampler::new(habitat, pre_sampler, self.lineages),
                &dispersal_sampler,
                coalescence_sampler,
                turnover_rate,
                speciation_probability,
                &event_sampler,
                self.resume_after.unwrap_or(NonNegativeF64::zero()),
            );

        if !exceptional_lineages.is_empty() {
            return Err(ResumeError::Sample(exceptional_lineages));
        }

        Ok((
            lineage_store,
            dispersal_sampler,
            event_sampler,
            active_lineage_sampler,
        ))
    }
}

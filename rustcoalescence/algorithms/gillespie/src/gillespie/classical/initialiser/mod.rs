use necsim_core::{
    cogs::{
        distribution::{Bernoulli, IndexUsize},
        ActiveLineageSampler, DispersalSampler, EmigrationExit, ImmigrationEntry,
        LocallyCoherentLineageStore, MathsCore, Rng, Samples,
    },
    reporter::Reporter,
};

use necsim_impls_no_std::cogs::{
    coalescence_sampler::unconditional::UnconditionalCoalescenceSampler,
    dispersal_sampler::in_memory::alias::InMemoryAliasDispersalSampler,
    event_sampler::unconditional::UnconditionalEventSampler, origin_sampler::TrustedOriginSampler,
    turnover_rate::uniform::UniformTurnoverRate,
};
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

pub mod fixup;
pub mod genesis;
pub mod resume;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::trait_duplication_in_bounds)]
pub trait ClassicalLineageStoreSampleInitialiser<
    M: MathsCore,
    G: Rng<M> + Samples<M, IndexUsize> + Samples<M, Bernoulli>,
    O: Scenario<M, G>,
    Error,
>
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        S: LocallyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        I: ImmigrationEntry<M>,
    >: ActiveLineageSampler<
        M,
        O::Habitat,
        G,
        S,
        X,
        Self::DispersalSampler,
        UnconditionalCoalescenceSampler<M, O::Habitat, S>,
        UniformTurnoverRate,
        O::SpeciationProbability,
        UnconditionalEventSampler<
            M,
            O::Habitat,
            G,
            S,
            X,
            Self::DispersalSampler,
            UnconditionalCoalescenceSampler<M, O::Habitat, S>,
            UniformTurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    #[allow(clippy::type_complexity)]
    fn init<
        'h,
        'p,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat>,
        X: EmigrationExit<M, O::Habitat, G, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<'p, Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
        local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}

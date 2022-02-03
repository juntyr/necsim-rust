use necsim_core::{
    cogs::{
        ActiveLineageSampler, DispersalSampler, EmigrationExit, ImmigrationEntry, LineageReference,
        LocallyCoherentLineageStore, MathsCore, RngCore,
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

#[allow(clippy::type_complexity)]
#[allow(clippy::module_name_repetitions)]
pub trait ClassicalLineageStoreSampleInitialiser<
    M: MathsCore,
    G: RngCore<M>,
    O: Scenario<M, G>,
    Error,
>
{
    type DispersalSampler: DispersalSampler<M, O::Habitat, G>;
    type ActiveLineageSampler<
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
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
        UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
        UniformTurnoverRate,
        O::SpeciationProbability,
        UnconditionalEventSampler<
            M,
            O::Habitat,
            G,
            R,
            S,
            X,
            Self::DispersalSampler,
            UnconditionalCoalescenceSampler<M, O::Habitat, R, S>,
            UniformTurnoverRate,
            O::SpeciationProbability,
        >,
        I,
    >;

    fn init<
        'h,
        T: TrustedOriginSampler<'h, M, Habitat = O::Habitat>,
        R: LineageReference<M, O::Habitat>,
        S: LocallyCoherentLineageStore<M, O::Habitat, R>,
        X: EmigrationExit<M, O::Habitat, G, R, S>,
        I: ImmigrationEntry<M>,
        Q: Reporter,
        P: LocalPartition<Q>,
    >(
        self,
        origin_sampler: T,
        dispersal_sampler: O::DispersalSampler<InMemoryAliasDispersalSampler<M, O::Habitat, G>>,
        local_partition: &mut P,
    ) -> Result<
        (
            S,
            Self::DispersalSampler,
            Self::ActiveLineageSampler<R, S, X, I>,
        ),
        Error,
    >
    where
        O::Habitat: 'h;
}

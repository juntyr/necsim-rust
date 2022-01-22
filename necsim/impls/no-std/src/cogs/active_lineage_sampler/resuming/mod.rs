use alloc::vec::Vec;
use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        ActiveLineageSampler, Backup, CoalescenceSampler, DispersalSampler, EmigrationExit,
        EventSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore, MathsCore,
        RngCore, SpeciationProbability, TurnoverRate,
    },
    lineage::Lineage,
};
use necsim_core_bond::PositiveF64;

mod sampler;

pub mod lineage;

#[derive(Debug)]
#[allow(clippy::type_complexity)]
pub struct RestartFixUpActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
    A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
> {
    inner: A,
    restart_time: PositiveF64,
    fixable_lineages: Vec<Lineage>,
    _marker: PhantomData<(M, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
        A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
    > RestartFixUpActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I, A>
{
    #[must_use]
    pub fn new(
        active_lineage_sampler: A,
        fixable_lineages: Vec<Lineage>,
        restart_time: PositiveF64,
    ) -> Self {
        Self {
            inner: active_lineage_sampler,
            restart_time,
            fixable_lineages,
            _marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: LineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: EventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
        A: ActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>,
    > Backup for RestartFixUpActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I, A>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            inner: self.inner.backup_unchecked(),
            restart_time: self.restart_time,
            fixable_lineages: self.fixable_lineages.clone(),
            _marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}
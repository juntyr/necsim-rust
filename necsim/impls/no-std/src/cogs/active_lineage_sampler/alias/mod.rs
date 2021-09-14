use core::marker::PhantomData;

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use necsim_core::{
    cogs::{
        Backup, CoalescenceSampler, DispersalSampler, EmigrationExit, GloballyCoherentLineageStore,
        Habitat, ImmigrationEntry, LineageReference, MathsCore, RngCore, SpeciationProbability,
        TurnoverRate,
    },
};

use crate::cogs::event_sampler::gillespie::{GillespieEventSampler, GillespiePartialSimulation};

use self::dynamic::DynamicAliasMethodSampler;

mod dynamic;
mod sampler;

#[allow(clippy::module_name_repetitions)]
#[allow(clippy::type_complexity)]
pub struct AliasActiveLineageSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: GloballyCoherentLineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
    E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
    I: ImmigrationEntry<M>,
> {
    alias_sampler: DynamicAliasMethodSampler<R>,
    number_active_lineages: usize,
    last_event_time: NonNegativeF64,
    marker: PhantomData<(M, H, G, R, S, X, D, C, T, N, E, I)>,
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > AliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    #[must_use]
    pub fn new(
        partial_simulation: &GillespiePartialSimulation<M, H, G, R, S, D, C, T, N>,
        event_sampler: &E,
    ) -> Self {
        let mut alias_sampler = DynamicAliasMethodSampler::new();
        let mut number_active_lineages: usize = 0;

        partial_simulation
            .lineage_store
            .iter_local_lineage_references()
            .for_each(|reference| {
                let location = partial_simulation.lineage_store[reference.clone()].indexed_location.location();
                let rate = partial_simulation.turnover_rate.get_turnover_rate_at_location(location, &partial_simulation.habitat);

                if let Ok(event_rate) = PositiveF64::new(rate.get()) {
                    alias_sampler.add_push(reference, event_rate);

                    number_active_lineages += 1;
                }
            });

        Self {
            alias_sampler,
            number_active_lineages,
            last_event_time: NonNegativeF64::zero(),
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > core::fmt::Debug for AliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("AliasActiveLineageSampler")
            .field("alias_sampler", &self.alias_sampler)
            .field("number_active_lineages", &self.number_active_lineages)
            .field("marker", &self.marker)
            .finish()
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        G: RngCore<M>,
        R: LineageReference<M, H>,
        S: GloballyCoherentLineageStore<M, H, R>,
        X: EmigrationExit<M, H, G, R, S>,
        D: DispersalSampler<M, H, G>,
        C: CoalescenceSampler<M, H, R, S>,
        T: TurnoverRate<M, H>,
        N: SpeciationProbability<M, H>,
        E: GillespieEventSampler<M, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<M>,
    > Backup for AliasActiveLineageSampler<M, H, G, R, S, X, D, C, T, N, E, I>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            alias_sampler: self.alias_sampler.clone(),
            number_active_lineages: self.number_active_lineages,
            last_event_time: self.last_event_time,
            marker: PhantomData::<(M, H, G, R, S, X, D, C, T, N, E, I)>,
        }
    }
}

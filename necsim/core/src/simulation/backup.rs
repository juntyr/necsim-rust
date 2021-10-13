use core::marker::PhantomData;

use crate::cogs::{
    backup::BackedUp, ActiveLineageSampler, Backup, CoalescenceSampler, DispersalSampler,
    EmigrationExit, EventSampler, F64Core, Habitat, ImmigrationEntry, LineageReference,
    LineageStore, RngCore, SpeciationProbability, TurnoverRate,
};

use super::Simulation;

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: EventSampler<F, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<F>,
        A: ActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>,
    > Backup for Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Simulation {
            f64_core: PhantomData::<F>,
            habitat: self.habitat.backup_unchecked(),
            lineage_reference: PhantomData::<R>,
            lineage_store: self.lineage_store.backup_unchecked(),
            emigration_exit: self.emigration_exit.backup_unchecked(),
            dispersal_sampler: self.dispersal_sampler.backup_unchecked(),
            coalescence_sampler: self.coalescence_sampler.backup_unchecked(),
            turnover_rate: self.turnover_rate.backup_unchecked(),
            speciation_probability: self.speciation_probability.backup_unchecked(),
            event_sampler: self.event_sampler.backup_unchecked(),
            active_lineage_sampler: self.active_lineage_sampler.backup_unchecked(),
            rng: self.rng.backup_unchecked(),
            immigration_entry: self.immigration_entry.backup_unchecked(),
            migration_balance: self.migration_balance,
        }
    }
}

impl<
        F: F64Core,
        H: Habitat<F>,
        G: RngCore<F>,
        R: LineageReference<F, H>,
        S: LineageStore<F, H, R>,
        X: EmigrationExit<F, H, G, R, S>,
        D: DispersalSampler<F, H, G>,
        C: CoalescenceSampler<F, H, R, S>,
        T: TurnoverRate<F, H>,
        N: SpeciationProbability<F, H>,
        E: EventSampler<F, H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry<F>,
        A: ActiveLineageSampler<F, H, G, R, S, X, D, C, T, N, E, I>,
    > BackedUp<Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A>>
{
    #[allow(clippy::type_complexity)]
    pub fn resume(&self) -> Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A> {
        unsafe {
            Simulation {
                f64_core: PhantomData::<F>,
                habitat: self.0.habitat.backup_unchecked(),
                lineage_reference: PhantomData::<R>,
                lineage_store: self.0.lineage_store.backup_unchecked(),
                emigration_exit: self.0.emigration_exit.backup_unchecked(),
                dispersal_sampler: self.0.dispersal_sampler.backup_unchecked(),
                coalescence_sampler: self.0.coalescence_sampler.backup_unchecked(),
                turnover_rate: self.0.turnover_rate.backup_unchecked(),
                speciation_probability: self.0.speciation_probability.backup_unchecked(),
                event_sampler: self.0.event_sampler.backup_unchecked(),
                active_lineage_sampler: self.0.active_lineage_sampler.backup_unchecked(),
                rng: self.0.rng.backup_unchecked(),
                immigration_entry: self.0.immigration_entry.backup_unchecked(),
                migration_balance: self.0.migration_balance,
            }
        }
    }
}

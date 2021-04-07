use core::marker::PhantomData;

use crate::cogs::{
    ActiveLineageSampler, BackedUp, Backup, CoalescenceSampler, DispersalSampler, EmigrationExit,
    EventSampler, Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore,
    SpeciationProbability, TurnoverRate,
};

use super::Simulation;

#[contract_trait]
impl<
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: EventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
        A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > Backup for Simulation<H, G, R, S, X, D, C, T, N, E, I, A>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Simulation {
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
        H: Habitat,
        G: RngCore,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, R, S>,
        D: DispersalSampler<H, G>,
        C: CoalescenceSampler<H, R, S>,
        T: TurnoverRate<H>,
        N: SpeciationProbability<H>,
        E: EventSampler<H, G, R, S, X, D, C, T, N>,
        I: ImmigrationEntry,
        A: ActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > BackedUp<Simulation<H, G, R, S, X, D, C, T, N, E, I, A>>
{
    pub fn resume(&self) -> Simulation<H, G, R, S, X, D, C, T, N, E, I, A> {
        unsafe {
            Simulation {
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

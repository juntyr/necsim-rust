mod backup;
mod builder;
mod process;

pub mod partial;

use core::num::Wrapping;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
        TurnoverRate, F64Core,
    },
    reporter::Reporter,
};

#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use builder::{Simulation, SimulationBuilder};
use necsim_core_bond::{NonNegativeF64, PositiveF64};

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
    > Simulation<F, H, G, R, S, X, D, C, T, N, E, I, A>
{
    pub fn is_done(&self) -> bool {
        self.active_lineage_sampler.number_active_lineages() == 0
            && self.immigration_entry.peek_next_immigration().is_none()
    }

    pub fn get_balanced_remaining_work(&self) -> Wrapping<u64> {
        let local_remaining =
            Wrapping(self.active_lineage_sampler().number_active_lineages() as u64);

        local_remaining + self.migration_balance
    }

    #[inline]
    pub fn simulate_incremental_early_stop<
        W: FnMut(&Self, u64, PositiveF64) -> bool,
        P: Reporter,
    >(
        &mut self,
        mut early_stop: W,
        reporter: &mut P,
    ) -> (NonNegativeF64, u64) {
        let mut steps = 0_u64;

        loop {
            reporter.report_progress(&self.get_balanced_remaining_work().0.into());

            let next_immigration_time = self
                .immigration_entry
                .peek_next_immigration()
                .map(|lineage| lineage.event_time);

            let self_ptr = self as *const Self;

            let old_rng = unsafe { self.rng.backup_unchecked() };
            let mut do_early_stop = false;

            let early_peek_stop = |next_event_time| {
                // Safety: We are only passing in an immutable reference
                do_early_stop = early_stop(unsafe { &*self_ptr }, steps, next_event_time);

                if do_early_stop {
                    return true;
                }

                if let Some(next_immigration_time) = next_immigration_time {
                    return next_immigration_time <= next_event_time;
                }

                false
            };

            if !self
                .simulate_and_report_local_step_or_early_stop_or_finish(reporter, early_peek_stop)
            {
                if do_early_stop {
                    // Early stop, reset the RNG to before the event time peek to eliminate side
                    // effects
                    break self.rng = old_rng;
                }

                // Check for migration as the alternative to finishing the simulation
                if let Some(migrating_lineage) =
                    self.immigration_entry_mut().next_optional_immigration()
                {
                    self.simulate_and_report_immigration_step(reporter, migrating_lineage);
                } else {
                    // Neither a local nor immigration event -> finish the simulation
                    break;
                }
            }

            steps += 1;
        }

        reporter.report_progress(&self.get_balanced_remaining_work().0.into());

        (self.active_lineage_sampler.get_last_event_time(), steps)
    }

    #[inline]
    pub fn simulate<P: Reporter>(mut self, reporter: &mut P) -> (NonNegativeF64, u64, G) {
        let (time, steps) = self.simulate_incremental_early_stop(|_, _, _| false, reporter);

        (time, steps, self.rng)
    }
}

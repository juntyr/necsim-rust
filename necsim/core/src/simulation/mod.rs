mod backup;
mod builder;
mod process;

pub mod partial;

use core::num::Wrapping;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore,
        OptionallyPeekableActiveLineageSampler, PeekableActiveLineageSampler, RngCore,
        SpeciationProbability, TurnoverRate,
    },
    lineage::MigratingLineage,
    reporter::{used::Unused, Reporter},
};

pub use builder::Simulation;

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
        A: PeekableActiveLineageSampler<H, G, R, S, X, D, C, T, N, E, I>,
    > Simulation<H, G, R, S, X, D, C, T, N, E, I, A>
{
    pub fn peek_time_of_next_event(&mut self) -> Option<f64> {
        let next_immigration_time = self
            .immigration_entry
            .peek_next_immigration()
            .map(|migrating_lineage| migrating_lineage.event_time);
        let next_local_time = self
            .active_lineage_sampler
            .peek_time_of_next_event(&mut self.rng)
            .ok();

        match (next_immigration_time, next_local_time) {
            (Some(next_immigration_time), Some(next_local_time)) => {
                Some(next_immigration_time.min(next_local_time))
            },
            (Some(next_event_time), _) | (_, Some(next_event_time)) => Some(next_event_time),
            (None, None) => None,
        }
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate_incremental_until_before<P: Reporter>(
        &mut self,
        max_time_exclusive: f64,
        reporter: &mut P,
    ) -> (f64, u64) {
        self.simulate_incremental_early_stop(
            |simulation, _| {
                simulation
                    .peek_time_of_next_event()
                    .map_or(true, |next_time| next_time >= max_time_exclusive)
            },
            reporter,
        )
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate_incremental_until<P: Reporter>(
        &mut self,
        max_time_inclusive: f64,
        reporter: &mut P,
    ) -> (f64, u64) {
        self.simulate_incremental_early_stop(
            |simulation, _| {
                simulation
                    .peek_time_of_next_event()
                    .map_or(true, |next_time| next_time > max_time_inclusive)
            },
            reporter,
        )
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
    > Simulation<H, G, R, S, X, D, C, T, N, E, I, A>
{
    pub fn get_balanced_remaining_work(&self) -> Wrapping<u64> {
        let local_remaining =
            Wrapping(self.active_lineage_sampler().number_active_lineages() as u64);

        local_remaining + self.migration_balance
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    fn simulate_incremental_early_stop<F: FnMut(&mut Self, u64) -> bool, P: Reporter>(
        &mut self,
        mut early_stop: F,
        reporter: &mut P,
    ) -> (f64, u64) {
        let mut steps = 0_u64;

        reporter.report_progress(Unused::new(&self.get_balanced_remaining_work().0));

        while !early_stop(self, steps) {
            // Peek the time of the next local event
            let optional_next_event_time = self.with_mut_split_active_lineage_sampler_and_rng(
                |active_lineage_sampler, _simulation, rng| {
                    active_lineage_sampler.peek_optional_time_of_next_event(rng)
                },
            );

            // Check if an immigration event has to be processed before the next local event
            if let Some(MigratingLineage {
                global_reference,
                dispersal_origin,
                dispersal_target,
                event_time: migration_event_time,
                coalescence_rng_sample,
            }) = self
                .immigration_entry_mut()
                .next_optional_immigration(optional_next_event_time)
            {
                process::immigration::simulate_and_report_immigration_step(
                    self,
                    reporter,
                    global_reference,
                    dispersal_origin,
                    dispersal_target,
                    migration_event_time,
                    coalescence_rng_sample,
                );
            } else if !process::local::simulate_and_report_local_step_or_finish(self, reporter) {
                reporter.report_progress(Unused::new(&self.get_balanced_remaining_work().0));

                break;
            }

            reporter.report_progress(Unused::new(&self.get_balanced_remaining_work().0));

            steps += 1;
        }

        (self.active_lineage_sampler.get_last_event_time(), steps)
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate_incremental_for<P: Reporter>(
        &mut self,
        max_steps: u64,
        reporter: &mut P,
    ) -> (f64, u64) {
        self.simulate_incremental_early_stop(|_, steps| steps >= max_steps, reporter)
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate_incremental_until_after<P: Reporter>(
        &mut self,
        min_time_inclusive: f64,
        reporter: &mut P,
    ) -> (f64, u64) {
        self.simulate_incremental_early_stop(
            |simulation, _| {
                simulation.active_lineage_sampler().get_last_event_time() >= min_time_inclusive
            },
            reporter,
        )
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate<P: Reporter>(mut self, reporter: &mut P) -> (f64, u64, G) {
        let (time, steps) = self.simulate_incremental_early_stop(|_, _| false, reporter);

        (time, steps, self.rng)
    }
}

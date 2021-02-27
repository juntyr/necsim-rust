mod backup;
mod builder;
mod process;

pub mod partial;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
    lineage::MigratingLineage,
    reporter::Reporter,
};

pub use builder::Simulation;

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, R, S>,
        E: EventSampler<H, G, N, D, R, S, X, C>,
        I: ImmigrationEntry,
        A: ActiveLineageSampler<H, G, N, D, R, S, X, C, E, I>,
    > Simulation<H, G, N, D, R, S, X, C, E, I, A>
{
    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    fn simulate_incremental_early_stop<F: FnMut(f64, u64) -> bool, P: Reporter>(
        &mut self,
        mut early_stop: F,
        reporter: &mut P,
    ) -> (f64, u64) {
        let mut steps = 0_u64;

        reporter.report_progress(self.active_lineage_sampler().number_active_lineages() as u64);

        while !early_stop(self.active_lineage_sampler.get_time_of_last_event(), steps) {
            // Peek the time of the next local event
            let optional_next_event_time = self.with_mut_split_active_lineage_sampler_and_rng(
                |active_lineage_sampler, _simulation, rng| {
                    active_lineage_sampler.peek_time_of_next_event(rng)
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
                reporter.report_progress(0_u64);

                break;
            }

            reporter.report_progress(self.active_lineage_sampler().number_active_lineages() as u64);

            steps += 1;
        }

        (self.active_lineage_sampler.get_time_of_last_event(), steps)
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
    pub fn simulate_incremental_until<P: Reporter>(
        &mut self,
        min_time: f64,
        reporter: &mut P,
    ) -> (f64, u64) {
        self.simulate_incremental_early_stop(|time, _| time >= min_time, reporter)
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate<P: Reporter>(mut self, reporter: &mut P) -> (f64, u64) {
        self.simulate_incremental_early_stop(|_, _| false, reporter)
    }
}

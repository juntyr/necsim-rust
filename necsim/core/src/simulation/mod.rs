mod builder;
mod process;

pub mod partial;

use crate::{
    cogs::{
        ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
        Habitat, ImmigrationEntry, LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
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
    #[debug_requires(max_steps > 0, "must run for at least one step")]
    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    #[inline]
    pub fn simulate_incremental<P: Reporter>(
        &mut self,
        max_steps: u64,
        reporter: &mut P,
    ) -> (f64, u64) {
        let mut steps = 0_u64;

        while steps < max_steps {
            // Peek the time of the next local event
            let optional_next_event_time = self.with_mut_split_active_lineage_sampler_and_rng(
                |active_lineage_sampler, _simulation, rng| {
                    active_lineage_sampler.peek_time_of_next_event(rng)
                },
            );

            // Check if an immigration event has to be processed before the next local event
            if let Some((
                global_reference,
                dispersal_origin,
                dispersal_target,
                migration_event_time,
                coalescence_rng_sample,
            )) = self
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
                break;
            }

            reporter.report_progress(self.active_lineage_sampler().number_active_lineages() as u64);

            steps += 1;
        }

        (self.active_lineage_sampler.get_time_of_last_event(), steps)
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    pub fn simulate<P: Reporter>(mut self, reporter: &mut P) -> (f64, u64) {
        let mut total_steps = 0_u64;

        let (mut final_time, mut new_steps) = self.simulate_incremental(u64::MAX, reporter);

        while new_steps > 0 {
            total_steps += new_steps;

            // Waiting for tuple destructuring RFC #2909 at:
            //   https://github.com/rust-lang/rust/pull/78748
            let (new_final_time, new_new_steps) = self.simulate_incremental(u64::MAX, reporter);
            final_time = new_final_time;
            new_steps = new_new_steps;
        }

        (final_time, total_steps)
    }
}

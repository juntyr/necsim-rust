mod builder;

pub mod partial;

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler,
    Habitat, LineageReference, LineageStore, RngCore, SpeciationProbability,
};

pub use builder::Simulation;

use crate::{event::EventType, reporter::Reporter};

impl<
        H: Habitat,
        G: RngCore,
        N: SpeciationProbability<H>,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        X: EmigrationExit<H, G, N, D, R, S>,
        C: CoalescenceSampler<H, G, R, S>,
        E: EventSampler<H, G, N, D, R, S, X, C>,
        A: ActiveLineageSampler<H, G, N, D, R, S, X, C, E>,
    > Simulation<H, G, N, D, R, S, X, C, E, A>
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

        while steps < max_steps
            && self.with_mut_split_active_lineage_sampler_and_rng(
                |active_lineage_sampler, simulation, rng| {
                    // Fetch the next `chosen_lineage` to be simulated with its
                    // `dispersal_origin` and `event_time`
                    active_lineage_sampler.with_next_active_lineage_indexed_location_event_time(
                        simulation,
                        rng,
                        |simulation, rng, chosen_lineage, dispersal_origin, event_time| {
                            // Sample the next `event` for the `chosen_lineage`
                            //  or emigrate the `chosen_lineage`
                            simulation.with_mut_split_event_sampler(
                                |event_sampler, simulation| {
                                    event_sampler.sample_event_for_lineage_at_indexed_location_time_or_emigrate(
                                        chosen_lineage,
                                        dispersal_origin,
                                        event_time,
                                        simulation,
                                        rng,
                                    )
                                },
                            ).and_then(|event| {
                                reporter.report_event(&event);

                                // In the event of dispersal without coalescence, the lineage remains
                                // active
                                if let EventType::Dispersal {
                                    target: dispersal_target,
                                    coalescence: None,
                                    ..
                                } = event.r#type() {
                                    Some(dispersal_target.clone())
                                } else { None }
                            })
                        },
                    )
                },
            )
        {
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

mod builder;

pub mod partial;

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
    LineageReference, LineageStore, RngCore,
};

pub use builder::Simulation;

use crate::event::EventType;
use crate::reporter::Reporter;

impl<
        H: Habitat,
        G: RngCore,
        D: DispersalSampler<H, G>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, G, R, S>,
        E: EventSampler<H, G, D, R, S, C>,
        A: ActiveLineageSampler<H, G, D, R, S, C, E>,
    > Simulation<H, G, D, R, S, C, E, A>
{
    #[debug_requires(max_steps > 0, "must run for at least one step")]
    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    pub fn simulate_incremental(
        &mut self,
        max_steps: usize,
        reporter: &mut impl Reporter<H, R>,
    ) -> (f64, usize) {
        let mut time = self
            .active_lineage_sampler
            .get_time_of_last_event(&self.lineage_store);
        let mut steps: usize = 0;

        self.with_mut_split_active_lineage_sampler_and_rng(
            |active_lineage_sampler, simulation, rng| {
                while let Some((chosen_lineage, dispersal_origin, event_time)) =
                    active_lineage_sampler
                        .pop_active_lineage_indexed_location_event_time(time, simulation, rng)
                {
                    let event = if cfg!(not(target_os = "cuda"))
                        && active_lineage_sampler.number_active_lineages() == 0
                    {
                        // Early stop iff only one active lineage is left
                        //  -> jump immediately to its speciation
                        simulation.with_split_event_sampler(|event_sampler, simulation| {
                            event_sampler.sample_final_speciation_event_for_lineage_after_time(
                                chosen_lineage,
                                time,
                                simulation,
                                rng,
                            )
                        })
                    } else {
                        simulation.with_split_event_sampler(|event_sampler, simulation| {
                            event_sampler.sample_event_for_lineage_at_indexed_location_time(
                                chosen_lineage,
                                dispersal_origin,
                                event_time,
                                simulation,
                                rng,
                            )
                        })
                    };

                    // Advance the simulation time
                    time = event.time();
                    steps += 1;

                    if let EventType::Dispersal {
                        target: dispersal_target,
                        coalescence: None,
                        ..
                    } = event.r#type()
                    {
                        // In the event of dispersal without coalescence, the lineage remains active
                        active_lineage_sampler.push_active_lineage_to_indexed_location(
                            event.lineage_reference().clone(),
                            dispersal_target.clone(),
                            time,
                            simulation,
                            rng,
                        );
                    }

                    reporter.report_event(&event);

                    // TODO: If reporters are ever to suggest an early stop, max_steps should become one
                    if steps >= max_steps {
                        break;
                    }
                }
            },
        );

        (time, steps)
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    pub fn simulate(mut self, reporter: &mut impl Reporter<H, R>) -> (f64, usize) {
        let mut total_steps: usize = 0;

        let (mut final_time, mut new_steps) = self.simulate_incremental(usize::MAX, reporter);

        while new_steps > 0 {
            total_steps += new_steps;

            // Waiting for tuple destructuring RFC #2909 at:
            //   https://github.com/rust-lang/rust/pull/78748
            let (new_final_time, new_new_steps) = self.simulate_incremental(usize::MAX, reporter);
            final_time = new_final_time;
            new_steps = new_new_steps;
        }

        (final_time, total_steps)
    }
}

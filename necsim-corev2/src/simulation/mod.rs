mod builder;

use crate::cogs::{
    ActiveLineageSampler, CoalescenceSampler, DispersalSampler, EventSampler, Habitat,
    LineageReference, LineageStore,
};

pub use builder::Simulation;

use crate::event::EventType;
use crate::reporter::Reporter;
use crate::rng::Rng;

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: CoalescenceSampler<H, R, S>,
        E: EventSampler<H, D, R, S, C>,
        A: ActiveLineageSampler<H, D, R, S, C, E>,
    > Simulation<H, D, R, S, C, E, A>
{
    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    pub fn simulate(
        mut self,
        rng: &mut impl Rng,
        reporter: &mut impl Reporter<H, R>,
    ) -> (f64, usize) {
        let mut time: f64 = 0.0;
        let mut steps: usize = 0;

        while let Some((chosen_lineage, event_time)) =
            A::pop_active_lineage_and_time_of_next_event(time, &mut self, rng)
        {
            let event = if self.active_lineage_sampler.number_active_lineages() == 0 {
                // Early stop iff only one active lineage is left
                //  -> jump immediately to its speciation
                self.event_sampler
                    .sample_final_speciation_event_for_lineage_after_time(
                        chosen_lineage,
                        time,
                        self.speciation_probability_per_generation,
                        rng,
                    )
            } else {
                self.event_sampler.sample_event_for_lineage_at_time(
                    chosen_lineage,
                    event_time,
                    self.speciation_probability_per_generation,
                    &self.habitat,
                    &self.dispersal_sampler,
                    &self.lineage_store,
                    &self.coalescence_sampler,
                    rng,
                )
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
                A::push_active_lineage_to_location(
                    event.lineage_reference().clone(),
                    dispersal_target.clone(),
                    time,
                    &mut self,
                    rng,
                );
            }

            reporter.report_event(&event);
        }

        (time, steps)
    }
}

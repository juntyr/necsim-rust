mod builder;
mod stages;
mod test;

use stages::{
    CoalescenceSampler, DispersalSampler, EventStage, Habitat, LineageReference, LineageSampler,
    ProbabilityStage,
};

use builder::Simulation;

use crate::reporter::Reporter;
use crate::rng::Rng;

impl<
        H: Habitat,
        D: DispersalSampler<H>,
        R: LineageReference<H>,
        L: LineageSampler<H, R>,
        C: CoalescenceSampler<H, R, L>,
        P: ProbabilityStage<H, D, R, L, C>,
        E: EventStage<H, D, R, L, C, P>,
    > Simulation<H, D, R, L, C, P, E>
{
    pub fn habitat(&self) -> &H {
        &self.habitat
    }

    pub fn dispersal_sampler(&self) -> &D {
        &self.dispersal_sampler
    }

    pub fn lineage_sampler(&self) -> &L {
        &self.lineage_sampler
    }

    pub fn coalescence_sampler(&self) -> &C {
        &self.coalescence_sampler
    }

    pub fn probability(&self) -> &P {
        &self.probability
    }

    pub fn event(&self) -> &E {
        &self.event
    }

    #[debug_ensures(ret.0 >= 0.0_f64, "returned time is non-negative")]
    pub fn simulate(&mut self, rng: &mut impl Rng, reporter: &mut impl Reporter) -> (f64, usize) {
        let mut time: f64 = 0.0;
        let mut steps: usize = 0;

        /*while let Some(event) = event_generator.generate_next_event(time, &mut self, rng) {
            time = event.time();
            steps += 1;

            reporter.report_event(&event);
        }*/

        (time, steps)
    }
}

mod settings;

use crate::{event_generator::EventGenerator, landscape::Landscape, reporter::Reporter, rng::Rng};

pub use settings::SimulationSettings;

pub struct Simulation(std::marker::PhantomData<()>);

impl Simulation {
    #[must_use]
    pub fn simulate(
        settings: &SimulationSettings<impl Landscape>,
        mut event_generator: impl EventGenerator,
        rng: &mut impl Rng,
        reporter: &mut impl Reporter,
    ) -> (f64, usize) {
        let mut time: f64 = 0.0;
        let mut steps: usize = 0;

        while let Some(event) = event_generator.generate_next_event(time, settings, rng) {
            time = event.time();
            steps += 1;

            reporter.report_event(&event);
        }

        (time, steps)
    }
}

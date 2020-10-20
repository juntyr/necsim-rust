use necsim_core::landscape::Landscape;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use super::event_type_sampler::unconditional_no_coalescence::UnconditionalNoCoalescenceEventTypeSampler;
use super::lineage_sampler::global_gillespie::GlobalGillespieStore;

mod event_generator;

pub struct GlobalGillespieUnconditionalEventGenerator {
    event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
    lineage_store: GlobalGillespieStore,
}

impl GlobalGillespieUnconditionalEventGenerator {
    pub fn new(settings: &SimulationSettings<impl Landscape>, rng: &mut impl Rng) -> Self {
        Self {
            event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
            lineage_store: GlobalGillespieStore::new(settings, rng),
        }
    }

    #[debug_ensures(ret >= 0.0_f64, "delta_time sample is non-negative")]
    fn sample_final_speciation_delta_time(
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> f64 {
        rng.sample_exponential(0.5_f64 * settings.speciation_probability_per_generation())
    }
}

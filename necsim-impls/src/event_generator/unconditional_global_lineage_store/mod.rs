use necsim_core::landscape::Landscape;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use super::event_type_sampler::unconditional_no_coalescence::UnconditionalNoCoalescenceEventTypeSampler;
use super::lineage_sampler::global_store::GlobalLineageStore;

mod event_generator;

pub struct GlobalLineageStoreUnconditionalEventGenerator {
    event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
    lineage_store: GlobalLineageStore,
}

impl GlobalLineageStoreUnconditionalEventGenerator {
    pub fn new(settings: &SimulationSettings<impl Landscape>, rng: &mut impl Rng) -> Self {
        Self {
            event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
            lineage_store: GlobalLineageStore::new(settings, rng),
        }
    }

    #[debug_ensures(ret >= 0.0_f64, "delta_time sample is non-negative")]
    fn sample_delta_time(&self, rng: &mut impl Rng) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * (self.lineage_store.number_active_lineages() + 1) as f64;

        rng.sample_exponential(lambda)
    }

    #[debug_ensures(ret >= 0.0_f64, "delta_time sample is non-negative")]
    fn sample_final_speciation_delta_time(
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> f64 {
        rng.sample_exponential(0.5_f64 * settings.speciation_probability_per_generation())
    }
}

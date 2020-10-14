use super::event_type_sampler::EventTypeSampler;

use super::event_type_sampler::unconditional_no_coalescence::UnconditionalNoCoalescenceEventTypeSampler;
use super::lineage_sampler::global_store::GlobalLineageStore;

use necsim_core::event_generator::{Event, EventGenerator, EventType};
use necsim_core::landscape::Landscape;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

pub struct GlobalLineageStoreUnconditionalEventGenerator {
    event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
    lineage_store: GlobalLineageStore,
}

impl EventGenerator for GlobalLineageStoreUnconditionalEventGenerator {
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event> {
        let chosen_active_lineage_reference =
            match self.lineage_store.pop_random_active_lineage_reference(rng) {
                Some(reference) => reference,
                None => return None,
            };

        if self.lineage_store.number_active_lineages() == 0 {
            // Early stop iff only one active lineage remains
            let event_time = time + Self::sample_final_speciation_delta_time(settings, rng);

            return Some(Event::new(event_time, EventType::Speciation));
        }

        let event_time = time + self.sample_delta_time(rng);
        let event_location = self.lineage_store[chosen_active_lineage_reference].location();

        let event_type_no_coalescence =
            self.event_type_sampler
                .sample_event_type_at_location(event_location, settings, rng);

        let event_type_with_coalescence = match event_type_no_coalescence {
            EventType::Speciation => EventType::Speciation,
            EventType::Dispersal {
                origin,
                target: dispersal_target,
                ..
            } => {
                let optional_coalescence =
                    self.lineage_store.sample_optional_coalescence_at_location(
                        &dispersal_target,
                        settings
                            .landscape()
                            .get_habitat_at_location(&dispersal_target),
                        rng,
                    );

                if optional_coalescence.is_none() {
                    // Apply the move to the chosen lineage

                    // TODO: assert success of dispersal in debug mode
                    //let pre = self.lineage_store.get_number_active_lineages_at_location(&dispersal_target);

                    self.lineage_store
                        .push_active_lineage_reference_at_location(
                            chosen_active_lineage_reference,
                            dispersal_target.clone(),
                        );

                    //let post = self.lineage_store.get_number_active_lineages_at_location(&dispersal_target);

                    //assert_eq!(post, pre + 1);
                    //assert_eq!(self.lineage_store[chosen_active_lineage_reference].location(), &dispersal_target);
                }

                EventType::Dispersal {
                    origin,
                    target: dispersal_target,
                    coalescence: optional_coalescence.is_some(),
                }
            }
        };

        Some(Event::new(event_time, event_type_with_coalescence))
    }
}

impl GlobalLineageStoreUnconditionalEventGenerator {
    pub fn new(landscape: &impl Landscape) -> Self {
        Self {
            event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
            lineage_store: GlobalLineageStore::new(landscape),
        }
    }

    fn sample_delta_time(&self, rng: &mut impl Rng) -> f64 {
        #[allow(clippy::cast_precision_loss)]
        let lambda = 0.5_f64 * (self.lineage_store.number_active_lineages() + 1) as f64;

        rng.sample_exponential(lambda)
    }

    fn sample_final_speciation_delta_time(
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> f64 {
        rng.sample_exponential(0.5_f64 * settings.speciation_probability_per_generation())
    }
}

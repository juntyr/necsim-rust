use super::event_type_sampler::EventTypeSampler;

use super::event_type_sampler::unconditional_no_coalescence::UnconditionalNoCoalescenceEventTypeSampler;
use super::lineage_sampler::global_store::{GlobalLineageStore, LineageReference};

use necsim_core::event_generator::{Event, EventGenerator, EventType};
use necsim_core::landscape::Landscape;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

pub struct GlobalLineageStoreUnconditionalEventGenerator {
    event_type_sampler: UnconditionalNoCoalescenceEventTypeSampler,
    lineage_store: GlobalLineageStore,
}

#[contract_trait]
impl EventGenerator<LineageReference> for GlobalLineageStoreUnconditionalEventGenerator {
    #[debug_ensures(
        (old(self.lineage_store.number_active_lineages()) == 0) == ret.is_none(),
        "returns None iff there are no active lineages left"
    )]
    #[debug_ensures(old(self.lineage_store.number_active_lineages()) == 1 ->
        ret.is_some() && match ret.as_ref().unwrap().r#type() {
            EventType::Speciation => true,
            _ => false,
        }, "last active lineage always speciates"
    )]
    // TODO: Check if speciation, lineage is no longer active and number active decreased
    // TODO: Check if dispersal, lineage is active, at location, and number active equal
    // TODO: Check if coalescence, lineage is no longer active, number active decreased and parent active and at location
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event<LineageReference>> {
        let chosen_active_lineage_reference =
            match self.lineage_store.pop_random_active_lineage_reference(rng) {
                Some(reference) => reference,
                None => return None,
            };

        if self.lineage_store.number_active_lineages() == 0 {
            // Early stop iff only one active lineage remains
            let event_time = time + Self::sample_final_speciation_delta_time(settings, rng);

            return Some(Event::new(
                event_time,
                chosen_active_lineage_reference,
                EventType::Speciation,
            ));
        }

        let event_time = time + self.sample_delta_time(rng);
        let event_location = self.lineage_store[chosen_active_lineage_reference].location();

        let event_type_no_coalescence: EventType<LineageReference> = self
            .event_type_sampler
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
                    self.lineage_store
                        .push_active_lineage_reference_at_location(
                            chosen_active_lineage_reference,
                            dispersal_target.clone(),
                        );
                }

                EventType::Dispersal {
                    origin,
                    target: dispersal_target,
                    coalescence: optional_coalescence,
                }
            }
        };

        Some(Event::new(
            event_time,
            chosen_active_lineage_reference,
            event_type_with_coalescence,
        ))
    }
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

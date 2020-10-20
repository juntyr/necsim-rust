use float_next_after::NextAfter;

use necsim_core::event_generator::{Event, EventGenerator, EventType};
use necsim_core::landscape::Landscape;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::event_type_sampler::EventTypeSampler;
use crate::event_generator::lineage_sampler::global_gillespie::LineageReference;

use super::GlobalGillespieUnconditionalEventGenerator;

#[contract_trait]
impl EventGenerator<LineageReference> for GlobalGillespieUnconditionalEventGenerator {
    #[debug_ensures(
        (old(self.lineage_store.number_active_lineages()) == 0) == ret.is_none(),
        "returns None iff there are no active lineages left"
    )]
    #[debug_ensures(old(self.lineage_store.number_active_lineages()) == 1 ->
        ret.is_some() && matches!(ret.as_ref().unwrap().r#type(), EventType::Speciation),
        "last active lineage always speciates"
    )]
    #[debug_ensures({
        let old_number_active_lineages = old(self.lineage_store.number_active_lineages());
        let new_number_active_lineages = self.lineage_store.number_active_lineages();

        match ret.as_ref() {
            Some(event) => match event.r#type() {
                EventType::Speciation | EventType::Dispersal {
                    coalescence: Some(_),
                    ..
                } => {
                    new_number_active_lineages == old_number_active_lineages - 1
                },
                EventType::Dispersal {
                    coalescence: None,
                    ..
                } => {
                    new_number_active_lineages == old_number_active_lineages
                },
            },
            None => new_number_active_lineages == old_number_active_lineages,
        }
    }, "an active lineage is only removed on speciation or coalescence")]
    #[debug_ensures(ret.is_some() -> {
        let event = ret.as_ref().unwrap();

        if let EventType::Dispersal {
            origin,
            target,
            coalescence: Some(parent_lineage),
        } = event.r#type() {
            self.lineage_store[*event.lineage_reference()].location() == origin &&
            self.lineage_store[*parent_lineage].location() == target
        } else {
            true
        }
    }, "coalesced lineage's parent is located at dispersal target")]
    #[debug_ensures(ret.is_some() -> {
        let event = ret.as_ref().unwrap();

        if let EventType::Dispersal {
            origin: _origin,
            target,
            coalescence: None,
        } = event.r#type() {
            self.lineage_store[*event.lineage_reference()].location() == target
        } else {
            true
        }
    }, "dispersed lineage has moved to dispersal target")]
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event<LineageReference>> {
        let (chosen_active_lineage_reference, chosen_event_time) = match self
            .lineage_store
            .pop_random_active_lineage_reference_and_event_time(rng)
        {
            Some(val) => val,
            None => return None,
        };

        if self.lineage_store.number_active_lineages() == 0 {
            // Early stop iff only one active lineage remains
            let event_time = time + Self::sample_final_speciation_delta_time(settings, rng);

            let unique_event_time: f64 = if event_time > time {
                event_time
            } else {
                event_time.next_after(f64::INFINITY)
            };

            return Some(Event::new(
                unique_event_time,
                chosen_active_lineage_reference,
                EventType::Speciation,
            ));
        }

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
                            chosen_event_time,
                            rng,
                        );
                }

                EventType::Dispersal {
                    origin,
                    target: dispersal_target,
                    coalescence: optional_coalescence,
                }
            }
        };

        let unique_event_time: f64 = if chosen_event_time > time {
            chosen_event_time
        } else {
            println!("Event time not increased: {} {}", time, chosen_event_time);

            time.next_after(f64::INFINITY)
        };

        Some(Event::new(
            unique_event_time,
            chosen_active_lineage_reference,
            event_type_with_coalescence,
        ))
    }
}

use float_next_after::NextAfter;

use necsim_core::event_generator::{Event, EventGenerator, EventType};
use necsim_core::landscape::Landscape;
use necsim_core::lineage::LineageReference;
use necsim_core::rng::Rng;
use necsim_core::simulation::SimulationSettings;

use crate::event_generator::coalescence_sampler::ConditionalCoalescenceSampler;
use crate::event_generator::event_type_sampler::EventTypeSampler;
use crate::event_generator::lineage_sampler::LineageSampler;

use super::UnconditionalGlobalEventGenerator;

#[contract_trait]
impl<L: LineageReference, S: LineageSampler<L> + ConditionalCoalescenceSampler<L>> EventGenerator<L>
    for UnconditionalGlobalEventGenerator<L, S>
{
    #[debug_ensures(
        (old(self.lineage_coalescence_sampler.number_active_lineages()) == 0) == ret.is_none(),
        "returns None iff there are no active lineages left"
    )]
    #[debug_ensures(old(self.lineage_coalescence_sampler.number_active_lineages()) == 1 ->
        ret.is_some() && matches!(ret.as_ref().unwrap().r#type(), EventType::Speciation),
        "last active lineage always speciates"
    )]
    #[debug_ensures({
        let old_number_active_lineages = old(
            self.lineage_coalescence_sampler.number_active_lineages()
        );
        let new_number_active_lineages = self.lineage_coalescence_sampler.number_active_lineages();

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
            self.lineage_coalescence_sampler[event.lineage_reference().clone()].location() == origin &&
            self.lineage_coalescence_sampler[parent_lineage.clone()].location() == target
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
            self.lineage_coalescence_sampler[event.lineage_reference().clone()].location() == target
        } else {
            true
        }
    }, "dispersed lineage has moved to dispersal target")]
    fn generate_next_event(
        &mut self,
        time: f64,
        settings: &SimulationSettings<impl Landscape>,
        rng: &mut impl Rng,
    ) -> Option<Event<L>> {
        let (chosen_active_lineage_reference, chosen_event_time) = match self
            .lineage_coalescence_sampler
            .pop_next_active_lineage_reference_and_event_time(time, rng)
        {
            Some(tuple) => tuple,
            None => return None,
        };

        if self.lineage_coalescence_sampler.number_active_lineages() == 0 {
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

        let event_location =
            self.lineage_coalescence_sampler[chosen_active_lineage_reference.clone()].location();

        let event_type: EventType<L> = self.event_type_sampler.sample_event_type_at_location(
            event_location,
            settings,
            &self.lineage_coalescence_sampler,
            rng,
        );

        if let EventType::Dispersal {
            target: ref dispersal_target,
            coalescence: None,
            ..
        } = event_type
        {
            self.lineage_coalescence_sampler
                .add_lineage_reference_to_location(
                    chosen_active_lineage_reference.clone(),
                    dispersal_target.clone(),
                    chosen_event_time,
                    rng,
                );
        }

        Some(Event::new(
            chosen_event_time,
            chosen_active_lineage_reference,
            event_type,
        ))
    }
}

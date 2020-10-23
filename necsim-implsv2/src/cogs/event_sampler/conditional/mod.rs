use necsim_corev2::cogs::{EventSampler, Habitat, LineageReference, LineageStore};
use necsim_corev2::event::{Event, EventType};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;

use crate::cogs::coalescence_sampler::conditional::ConditionalCoalescenceSampler;
use crate::cogs::dispersal_sampler::separable::SeparableDispersalSampler;

use super::final_speciation_event;

mod probability;

use probability::ProbabilityAtLocation;

#[allow(clippy::module_name_repetitions)]
pub struct ConditionalEventSampler;

#[contract_trait]
impl<
        H: Habitat,
        D: SeparableDispersalSampler<H>,
        R: LineageReference<H>,
        S: LineageStore<H, R>,
        C: ConditionalCoalescenceSampler<H, R, S>,
    > EventSampler<H, D, R, S, C> for ConditionalEventSampler
{
    #[must_use]
    #[debug_ensures(match &ret.r#type() {
        EventType::Speciation => true,
        EventType::Dispersal {
            origin,
            target,
            coalescence,
            ..
        } => coalescence.is_some() == (origin == target),
    }, "always coalesces on self-dispersal, never coalesces on out-dispersal")]
    fn sample_event_for_lineage_at_time(
        &self,
        lineage_reference: R,
        event_time: f64,
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        coalescence_sampler: &C,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let dispersal_origin = lineage_store[lineage_reference.clone()].location();

        let probability_at_location = ProbabilityAtLocation::new(
            dispersal_origin,
            speciation_probability_per_generation,
            habitat,
            dispersal_sampler,
            lineage_store,
            false, // lineage_reference was popped from the store
            coalescence_sampler,
        );

        let event_sample = probability_at_location.total() * rng.sample_uniform();

        let event_type = if event_sample < probability_at_location.speciation() {
            EventType::Speciation
        } else if event_sample
            < (probability_at_location.speciation() + probability_at_location.out_dispersal())
        {
            let dispersal_target =
                dispersal_sampler.sample_non_self_dispersal_from_location(dispersal_origin, rng);

            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                target: dispersal_target,
                coalescence: coalescence_sampler.sample_optional_coalescence_at_location(
                    dispersal_origin,
                    habitat,
                    lineage_store,
                    rng,
                ),
                _marker: std::marker::PhantomData::<H>,
            }
        } else {
            EventType::Dispersal {
                origin: dispersal_origin.clone(),
                target: dispersal_origin.clone(),
                coalescence: Some(coalescence_sampler.sample_coalescence_at_location(
                    dispersal_origin,
                    lineage_store,
                    rng,
                )),
                _marker: std::marker::PhantomData::<H>,
            }
        };

        Event::new(event_time, lineage_reference, event_type)
    }

    #[must_use]
    fn sample_final_speciation_event_for_lineage_after_time(
        &self,
        lineage_reference: R,
        time: f64,
        speciation_probability_per_generation: f64,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        final_speciation_event::sample_final_speciation_event_for_lineage_after_time(
            lineage_reference,
            time,
            speciation_probability_per_generation,
            rng,
        )
    }

    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        lineage_store_includes_self: bool,
        coalescence_sampler: &C,
    ) -> f64 {
        let probability_at_location = ProbabilityAtLocation::new(
            location,
            speciation_probability_per_generation,
            habitat,
            dispersal_sampler,
            lineage_store,
            lineage_store_includes_self,
            coalescence_sampler,
        );

        #[allow(clippy::cast_precision_loss)]
        let population = (lineage_store
            .get_active_lineages_at_location(location)
            .len()
            + usize::from(!lineage_store_includes_self)) as f64;

        probability_at_location.total() * population * 0.5_f64
    }
}

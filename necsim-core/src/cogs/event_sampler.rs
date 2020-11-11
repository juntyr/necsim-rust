use float_next_after::NextAfter;

use super::{
    CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore, RngCore,
};
use crate::event::{Event, EventType};
use crate::landscape::IndexedLocation;
use crate::simulation::partial::event_sampler::PartialSimulation;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
>: core::fmt::Debug
{
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time >= 0.0_f64, "event time is non-negative")]
    #[debug_requires(
        *simulation.speciation_probability_per_generation >= 0.0_f64 &&
        *simulation.speciation_probability_per_generation <= 1.0_f64,
        "speciation_probability_per_generation is a probability"
    )]
    #[debug_ensures(
        ret.lineage_reference() == &old(lineage_reference.clone()),
        "event occurs for lineage_reference"
    )]
    #[debug_ensures(ret.time() == event_time, "event occurs at event_time")]
    fn sample_event_for_lineage_at_indexed_location_time(
        &self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        event_time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        rng: &mut G,
    ) -> Event<H, R>;

    #[must_use]
    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_requires(
        *simulation.speciation_probability_per_generation >= 0.0_f64 &&
        *simulation.speciation_probability_per_generation <= 1.0_f64,
        "speciation_probability_per_generation is a probability"
    )]
    #[debug_ensures(
        ret.lineage_reference() == &old(lineage_reference.clone()),
        "event occurs for lineage_reference"
    )]
    #[debug_ensures(ret.time() > time, "event occurs after time")]
    #[debug_ensures(matches!(ret.r#type(), EventType::Speciation), "always samples speciation event")]
    fn sample_final_speciation_event_for_lineage_after_time(
        &self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        rng: &mut G,
    ) -> Event<H, R> {
        use crate::cogs::RngSampler;

        let delta_time =
            rng.sample_exponential(simulation.speciation_probability_per_generation * 0.5_f64);

        let event_time = time + delta_time;

        Event::new(
            indexed_location,
            if event_time > time {
                event_time
            } else {
                time.next_after(f64::INFINITY)
            },
            lineage_reference,
            EventType::Speciation,
        )
    }
}

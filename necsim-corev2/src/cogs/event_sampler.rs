use float_next_after::NextAfter;

use super::{CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore};
use crate::event::{Event, EventType};
use crate::rng::Rng;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
>
{
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time >= 0.0_f64, "event time is non-negative")]
    #[debug_requires(
        speciation_probability_per_generation >= 0.0_f64 &&
        speciation_probability_per_generation <= 1.0_f64,
        "speciation_probability_per_generation is a probability"
    )]
    #[debug_ensures(
        ret.lineage_reference() == &old(lineage_reference.clone()),
        "event occurs for lineage_reference"
    )]
    #[debug_ensures(ret.time() == event_time, "event occurs at event_time")]
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
    ) -> Event<H, R>;

    #[must_use]
    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_requires(
        speciation_probability_per_generation >= 0.0_f64 &&
        speciation_probability_per_generation <= 1.0_f64,
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
        time: f64,
        speciation_probability_per_generation: f64,
        rng: &mut impl Rng,
    ) -> Event<H, R> {
        let delta_time = rng.sample_exponential(speciation_probability_per_generation * 0.5_f64);

        let event_time = time + delta_time;

        Event::new(
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

use float_next_after::NextAfter;

use necsim_corev2::cogs::{Habitat, LineageReference};
use necsim_corev2::event::{Event, EventType};
use necsim_corev2::rng::Rng;

#[must_use]
pub fn sample_final_speciation_event_for_lineage_after_time<H: Habitat, R: LineageReference<H>>(
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
            event_time.next_after(f64::INFINITY)
        },
        lineage_reference,
        EventType::Speciation,
    )
}

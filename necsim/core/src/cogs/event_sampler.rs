use super::{
    CoalescenceSampler, DispersalSampler, Habitat, LineageReference, LineageStore, RngCore,
};
use crate::{
    event::Event, landscape::IndexedLocation, simulation::partial::event_sampler::PartialSimulation,
};

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
}

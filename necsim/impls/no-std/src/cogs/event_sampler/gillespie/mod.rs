pub mod conditional;
pub mod unconditional;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference,
        LineageStore, RngCore,
    },
    landscape::Location,
    simulation::partial::event_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
pub trait GillespieEventSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
>: EventSampler<H, G, D, R, S, C>
{
    #[must_use]
    #[debug_requires(
        simulation.speciation_probability_per_generation >= 0.0_f64 &&
        simulation.speciation_probability_per_generation <= 1.0_f64,
        "speciation_probability_per_generation is a probability"
    )]
    #[debug_ensures(ret >= 0.0_f64, "returns a rate")]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, D, R, S, C>,
        lineage_store_includes_self: bool,
    ) -> f64;
}

pub mod conditional;
pub mod unconditional;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, RngCore, SpeciationProbability,
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
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, N, D, R, S>,
    C: CoalescenceSampler<H, R, S>,
>: EventSampler<H, G, N, D, R, S, X, C>
{
    #[must_use]
    #[debug_ensures(ret >= 0.0_f64, "returns a rate")]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, N, D, R, S, X, C>,
        lineage_store_includes_self: bool,
    ) -> f64;
}

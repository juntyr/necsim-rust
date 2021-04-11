pub mod conditional;
pub mod unconditional;

use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, RngCore, SpeciationProbability, TurnoverRate,
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
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    X: EmigrationExit<H, G, R, S>,
    D: DispersalSampler<H, G>,
    C: CoalescenceSampler<H, R, S>,
    T: TurnoverRate<H>,
    N: SpeciationProbability<H>,
>: EventSampler<H, G, R, S, X, D, C, T, N>
{
    /// Pre: all lineages must be active in the lineage store
    #[must_use]
    #[debug_ensures(ret >= 0.0_f64, "returns a rate")]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        simulation: &PartialSimulation<H, G, R, S, X, D, C, T, N>,
    ) -> f64;
}

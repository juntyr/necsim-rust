pub mod conditional;
pub mod unconditional;

use necsim_corev2::landscape::Location;

use necsim_corev2::cogs::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions)]
pub trait GillespieEventSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
>: EventSampler<H, D, R, S, C>
{
    #[must_use]
    #[debug_requires(
        speciation_probability_per_generation >= 0.0_f64 &&
        speciation_probability_per_generation <= 1.0_f64,
        "speciation_probability_per_generation is a probability"
    )]
    #[debug_ensures(ret >= 0.0_f64, "returns a rate")]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        speciation_probability_per_generation: f64,
        habitat: &H,
        dispersal_sampler: &D,
        lineage_store: &S,
        lineage_store_includes_self: bool,
        coalescence_sampler: &C,
    ) -> f64;
}

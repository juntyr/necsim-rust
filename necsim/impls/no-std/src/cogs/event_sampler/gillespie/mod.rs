use necsim_core::{
    cogs::{
        CoalescenceSampler, DispersalSampler, EmigrationExit, EventSampler, Habitat,
        LineageReference, LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate,
    },
    landscape::Location,
};
use necsim_core_bond::NonNegativeF64;

pub mod conditional;
pub mod unconditional;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
#[allow(clippy::module_name_repetitions, clippy::too_many_arguments)]
pub trait GillespieEventSampler<
    M: MathsCore,
    H: Habitat<M>,
    G: RngCore<M>,
    R: LineageReference<M, H>,
    S: LineageStore<M, H, R>,
    X: EmigrationExit<M, H, G, R, S>,
    D: DispersalSampler<M, H, G>,
    C: CoalescenceSampler<M, H, R, S>,
    T: TurnoverRate<M, H>,
    N: SpeciationProbability<M, H>,
>: EventSampler<M, H, G, R, S, X, D, C, T, N>
{
    /// Pre: all lineages must be active in the lineage store
    #[must_use]
    fn get_event_rate_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        dispersal_sampler: &D,
        coalescence_sampler: &C,
        turnover_rate: &T,
        speciation_probability: &N,
    ) -> NonNegativeF64;
}

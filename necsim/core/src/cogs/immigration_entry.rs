use crate::{
    cogs::{
        DispersalSampler, Habitat, LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
    simulation::partial::migration::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ImmigrationEntry<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
>: core::fmt::Debug
{
    #[must_use]
    // TODO: Ensures that if Some, R has been added to the lineage store
    // TODO: Ensures that if Some, R's last event time is <= next_event_time
    fn next_optional_immigration(
        &mut self,
        next_event_time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S>,
        rng: &mut G,
    ) -> Option<R>;
}

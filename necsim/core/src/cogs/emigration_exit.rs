use crate::{
    cogs::{
        DispersalSampler, Habitat, LineageReference, LineageStore, RngCore, SpeciationProbability,
    },
    landscape::{IndexedLocation, Location},
    simulation::partial::migration::PartialSimulation,
};

// TODO: Should the emigration exit handle the removal from the store,
//       or should that be up to the active lineage sampler when no event is
// returned?

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait EmigrationExit<
    H: Habitat,
    G: RngCore,
    N: SpeciationProbability<H>,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
>: core::fmt::Debug
{
    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_requires(event_time >= 0.0_f64, "event time is non-negative")]
    #[debug_ensures(match &ret {
        Some((
            ret_lineage_reference,
            ret_dispersal_origin,
            ret_dispersal_target,
            ret_event_time,
        )) => {
            ret_lineage_reference == &old(lineage_reference.clone()) &&
            ret_dispersal_origin == &old(dispersal_origin.clone()) &&
            ret_dispersal_target == &old(dispersal_target.clone()) &&
            ret_event_time == &old(event_time)
        },
        None => true,
    }, "if ret is Some, it returns the input parameters unchanged")]
    // TODO: Ensures that if None, that the lineage reference has been removed from
    // the lineage store TODO: Ensures that lineage only emigrates iff dispersal
    // target outside local chunk
    fn optionally_emigrate(
        &mut self,
        lineage_reference: R,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        event_time: f64,
        simulation: &mut PartialSimulation<H, G, N, D, R, S>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, Location, f64)>;
}

use necsim_core_bond::{NonNegativeF64, PositiveF64};

use crate::{
    cogs::{Habitat, LineageReference, LineageStore, RngCore},
    landscape::{IndexedLocation, Location},
    simulation::partial::emigration_exit::PartialSimulation,
};

#[allow(
    clippy::inline_always,
    clippy::inline_fn_without_body,
    clippy::too_many_arguments
)]
#[contract_trait]
pub trait EmigrationExit<H: Habitat, G: RngCore, R: LineageReference<H>, S: LineageStore<H, R>>:
    crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_ensures(match &ret {
        Some((
            ret_lineage_reference,
            ret_dispersal_origin,
            ret_dispersal_target,
            ret_prior_time,
            ret_event_time,
        )) => {
            ret_lineage_reference == &old(lineage_reference.clone()) &&
            ret_dispersal_origin == &old(dispersal_origin.clone()) &&
            ret_dispersal_target == &old(dispersal_target.clone()) &&
            ret_prior_time == &old(prior_time) &&
            ret_event_time == &old(event_time)
        },
        None => true,
    }, "if ret is Some, it returns the input parameters unchanged")]
    #[debug_ensures(if ret.as_ref().is_none() {
        simulation.lineage_store.get(old(lineage_reference.clone())).is_none()
    } else { true }, "if ret is None, lineage_reference has been removed from the lineage store")]
    fn optionally_emigrate(
        &mut self,
        lineage_reference: R,
        dispersal_origin: IndexedLocation,
        dispersal_target: Location,
        prior_time: NonNegativeF64,
        event_time: PositiveF64,
        simulation: &mut PartialSimulation<H, G, R, S>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, Location, NonNegativeF64, PositiveF64)>;
}

use super::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
    RngCore,
};

use crate::{
    landscape::IndexedLocation, simulation::partial::active_lineager_sampler::PartialSimulation,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ActiveLineageSampler<
    H: Habitat,
    G: RngCore,
    D: DispersalSampler<H, G>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, G, R, S>,
    E: EventSampler<H, G, D, R, S, C>,
>: core::fmt::Debug
{
    #[must_use]
    fn number_active_lineages(&self) -> usize;

    #[must_use]
    #[debug_ensures(ret >= 0.0_f64, "last event time is non-negative")]
    fn get_time_of_last_event(&self, lineage_store: &S) -> f64;

    #[must_use]
    #[allow(clippy::float_cmp)]
    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_ensures(match ret {
        Some(_) => {
            self.number_active_lineages() ==
            old(self.number_active_lineages()) - 1
        },
        None => old(self.number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().2 > time,
        "event occurs later than time"
    )]
    #[debug_ensures(match ret {
        None => true,
        Some((ref reference, ref _location, event_time)) => {
            simulation.lineage_store[reference.clone()].time_of_last_event() == event_time
        },
    }, "updates the time of the last event of the returned lineage to the time of the event")]
    #[debug_ensures(match ret {
        None => true,
        Some((ref _reference, ref _location, event_time)) => {
            self.get_time_of_last_event(simulation.lineage_store) == event_time
        },
    }, "updates the time of the last event")]
    fn pop_active_lineage_indexed_location_event_time(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<H, G, D, R, S, C, E>,
        rng: &mut G,
    ) -> Option<(R, IndexedLocation, f64)>;

    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    // #[debug_requires(( TODO: How can we assert this only for coherent lineage
    // stores? simulation.lineage_store.get_active_lineages_at_location(&
    // location).len() < (simulation.habitat.get_habitat_at_location(&location)
    // as usize) ), "location has habitat capacity for the lineage")]
    fn push_active_lineage_to_indexed_location(
        &mut self,
        lineage_reference: R,
        indexed_location: IndexedLocation,
        time: f64,
        simulation: &mut PartialSimulation<H, G, D, R, S, C, E>,
        rng: &mut G,
    );
}

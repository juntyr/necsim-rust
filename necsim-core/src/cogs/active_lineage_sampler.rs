use super::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};

use crate::landscape::Location;
use crate::rng::Rng;
use crate::simulation::partial::active_lineager_sampler::PartialSimulation;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait ActiveLineageSampler<
    H: Habitat,
    D: DispersalSampler<H>,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
    C: CoalescenceSampler<H, R, S>,
    E: EventSampler<H, D, R, S, C>,
>: Sized
{
    #[must_use]
    fn number_active_lineages(&self) -> usize;

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
        ret.is_some() -> ret.as_ref().unwrap().1 > time,
        "event occurs later than time"
    )]
    #[debug_ensures(match ret {
        None => true,
        Some((ref reference, event_time)) => {
            simulation.lineage_store[reference.clone()].time_of_last_event() == event_time
        },
    }, "updates the time of the last event of the returned lineage to the time of the event")]
    fn pop_active_lineage_and_time_of_next_event(
        &mut self,
        time: f64,
        simulation: &mut PartialSimulation<H, D, R, S, C, E>,
        rng: &mut impl Rng,
    ) -> Option<(R, f64)>;

    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_requires((
        simulation.lineage_store.get_active_lineages_at_location(&location).len() <
        (simulation.habitat.get_habitat_at_location(&location) as usize)
    ), "location has habitat capacity for the lineage")]
    fn push_active_lineage_to_location(
        &mut self,
        lineage_reference: R,
        location: Location,
        time: f64,
        simulation: &mut PartialSimulation<H, D, R, S, C, E>,
        rng: &mut impl Rng,
    );
}

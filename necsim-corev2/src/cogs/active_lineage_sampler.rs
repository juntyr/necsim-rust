use super::{
    CoalescenceSampler, DispersalSampler, EventSampler, Habitat, LineageReference, LineageStore,
};

use crate::landscape::Location;
use crate::rng::Rng;
use crate::simulation::Simulation;

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
    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    #[debug_ensures(match ret {
        Some(_) => {
            simulation.active_lineage_sampler().number_active_lineages() ==
            old(simulation.active_lineage_sampler().number_active_lineages()) - 1
        },
        None => old(simulation.active_lineage_sampler().number_active_lineages()) == 0,
    }, "removes an active lineage if some left")]
    #[debug_ensures(
        ret.is_some() -> ret.as_ref().unwrap().1 > time,
        "event occurs later than time"
    )]
    fn pop_active_lineage_and_time_of_next_event(
        time: f64,
        simulation: &mut Simulation<H, D, R, S, C, E, Self>,
        rng: &mut impl Rng,
    ) -> Option<(R, f64)>;

    #[debug_requires(time >= 0.0_f64, "time is non-negative")]
    fn push_active_lineage_to_location(
        lineage_reference: R,
        location: Location,
        time: f64,
        simulation: &mut Simulation<H, D, R, S, C, E, Self>,
        rng: &mut impl Rng,
    );
}

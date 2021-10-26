use necsim_core::{
    cogs::{Habitat, MathsCore, PrimeableRng, TurnoverRate},
    landscape::IndexedLocation,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::cogs::active_lineage_sampler::independent::event_time_sampler::EventTimeSampler;

use crate::clock;

#[inline]
#[allow(clippy::needless_pass_by_value)]
pub fn exponential_inter_event_times<
    M: MathsCore,
    H: Habitat<M>,
    G: PrimeableRng<M>,
    T: TurnoverRate<M, H>,
    E: EventTimeSampler<M, H, G, T>,
>(
    habitat: H,
    mut rng: G,
    turnover_rate: T,
    event_time_sampler: E,
    indexed_location: IndexedLocation,
    limit: u128,
) -> (u64, u64) {
    let mut last_event_time = NonNegativeF64::zero();

    let time_start = clock::timer_ns();
    let cycle_start = clock::counter();

    for _ in 0..limit {
        let next_event_time = event_time_sampler.next_event_time_at_indexed_location_weakly_after(
            &indexed_location,
            last_event_time,
            &habitat,
            &mut rng,
            &turnover_rate,
        );

        last_event_time = next_event_time;
    }

    let cycle_finish = clock::counter();
    let time_finish = clock::timer_ns();

    (
        time_finish.wrapping_sub(time_start),
        cycle_finish.wrapping_sub(cycle_start),
    )
}

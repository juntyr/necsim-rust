use core::{
    num::NonZeroU32,
    sync::atomic::{AtomicU64, Ordering},
};

use necsim_core::{
    cogs::SeedableRng,
    landscape::{IndexedLocation, Location},
};
use necsim_core_bond::{OffByOneU32, PositiveF64};

use necsim_impls_cuda::cogs::maths::NvptxMathsCore;
use necsim_impls_no_std::cogs::{
    active_lineage_sampler::independent::event_time_sampler::EventTimeSampler,
    habitat::non_spatial::NonSpatialHabitat, rng::wyhash::WyHash,
};

use crate::{sample, UniformTurnoverRate};

#[inline]
#[allow(dead_code)]
pub fn inter_event_times<
    E: EventTimeSampler<
        NvptxMathsCore,
        NonSpatialHabitat<NvptxMathsCore>,
        WyHash<NvptxMathsCore>,
        UniformTurnoverRate,
    >,
>(
    event_time_sampler: E,
    seed: u64,
    lambda: PositiveF64,
    limit: u128,
    total_cycles_sum: &AtomicU64,
    total_time_sum: &AtomicU64,
) {
    let habitat = NonSpatialHabitat::new((OffByOneU32::one(), OffByOneU32::one()), unsafe {
        NonZeroU32::new_unchecked(1)
    });
    let rng = WyHash::seed_from_u64(seed + (rust_cuda::device::utils::index() as u64));
    let turnover_rate = UniformTurnoverRate::new(lambda);
    let indexed_location = IndexedLocation::new(Location::new(0, 0), 0);

    let (cycles, time) = sample::exponential_inter_event_times(
        habitat,
        rng,
        turnover_rate,
        event_time_sampler,
        indexed_location,
        limit,
    );

    total_cycles_sum.fetch_add(cycles, Ordering::Relaxed);
    total_time_sum.fetch_add(time, Ordering::Relaxed);
}

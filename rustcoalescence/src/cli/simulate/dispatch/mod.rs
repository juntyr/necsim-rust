#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod algorithm_scenario;
#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod partitioning;
#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod rng;

#[cfg(not(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
)))]
mod fallback;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
pub(super) use partitioning::dispatch;

#[cfg(not(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
)))]
pub(super) use fallback::dispatch;

#[cfg(any(
    feature = "gillespie-algorithms",
    feature = "independent-algorithm",
    feature = "cuda-algorithm"
))]
mod valid;

#[cfg(not(any(
    feature = "gillespie-algorithms",
    feature = "independent-algorithm",
    feature = "cuda-algorithm"
)))]
mod fallback;

#[cfg(any(
    feature = "gillespie-algorithms",
    feature = "independent-algorithm",
    feature = "cuda-algorithm"
))]
pub(super) use valid::dispatch;

#[cfg(not(any(
    feature = "gillespie-algorithms",
    feature = "independent-algorithm",
    feature = "cuda-algorithm"
)))]
pub(super) use fallback::dispatch;

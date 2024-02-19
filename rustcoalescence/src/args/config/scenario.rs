use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub enum Scenario {
    #[cfg(feature = "spatially-explicit-uniform-turnover-scenario")]
    #[serde(alias = "SpatiallyExplicit")]
    SpatiallyExplicitUniformTurnover(
        rustcoalescence_scenarios::spatially_explicit::SpatiallyExplicitUniformTurnoverArguments,
    ),
    #[cfg(feature = "spatially-explicit-turnover-map-scenario")]
    SpatiallyExplicitTurnoverMap(
        rustcoalescence_scenarios::spatially_explicit::SpatiallyExplicitTurnoverMapArguments,
    ),
    #[cfg(feature = "non-spatial-scenario")]
    NonSpatial(rustcoalescence_scenarios::non_spatial::NonSpatialArguments),
    #[cfg(feature = "spatially-implicit-scenario")]
    SpatiallyImplicit(rustcoalescence_scenarios::spatially_implicit::SpatiallyImplicitArguments),
    #[cfg(feature = "almost-infinite-scenario")]
    AlmostInfinite(rustcoalescence_scenarios::almost_infinite::AlmostInfiniteArguments),
    #[cfg(feature = "clark-scenario")]
    Clark(rustcoalescence_scenarios::clark::ClarkArguments),
    #[cfg(feature = "wrapping-noise-scenario")]
    WrappingNoise(rustcoalescence_scenarios::wrapping_noise::WrappingNoiseArguments),
}

mod maps;
mod turnover;

#[cfg(feature = "spatially-explicit-turnover-map")]
#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use turnover::map::{
    SpatiallyExplicitTurnoverMapArguments, SpatiallyExplicitTurnoverMapScenario,
};

#[cfg(feature = "spatially-explicit-uniform-turnover")]
#[allow(clippy::useless_attribute, clippy::module_name_repetitions)]
pub use turnover::uniform::{
    SpatiallyExplicitUniformTurnoverArguments, SpatiallyExplicitUniformTurnoverScenario,
};

use serde::{Deserialize, Serialize};

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteArguments,
    non_spatial::NonSpatialArguments,
    spatially_explicit::{
        SpatiallyExplicitTurnoverMapArguments, SpatiallyExplicitUniformTurnoverArguments,
    },
    spatially_implicit::SpatiallyImplicitArguments,
};

#[derive(Debug, Serialize, Deserialize)]
pub enum Scenario {
    #[serde(alias = "SpatiallyExplicit")]
    SpatiallyExplicitUniformTurnover(SpatiallyExplicitUniformTurnoverArguments),
    SpatiallyExplicitTurnoverMap(SpatiallyExplicitTurnoverMapArguments),
    NonSpatial(NonSpatialArguments),
    SpatiallyImplicit(SpatiallyImplicitArguments),
    AlmostInfinite(AlmostInfiniteArguments),
}

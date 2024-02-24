use either::Either;
use serde::{Deserialize, Serialize};

use necsim_core::landscape::{LandscapeExtent, Location};
#[cfg(feature = "almost-infinite-normal-dispersal")]
use necsim_core_bond::NonNegativeF64;
#[cfg(feature = "almost-infinite-clark2dt-dispersal")]
use necsim_core_bond::PositiveF64;

#[cfg(feature = "almost-infinite-clark2dt-dispersal")]
pub mod clark2dt;
#[cfg(feature = "almost-infinite-normal-dispersal")]
pub mod normal;

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(rename = "AlmostInfinite")]
pub struct AlmostInfiniteArguments {
    sample: Sample,
    dispersal: Dispersal,
}

#[cfg(feature = "almost-infinite-normal-dispersal")]
type NormalDispersalArguments = normal::AlmostInfiniteNormalDispersalArguments;
#[cfg(not(feature = "almost-infinite-normal-dispersal"))]
type NormalDispersalArguments = !;

#[cfg(feature = "almost-infinite-clark2dt-dispersal")]
type Clark2DtDispersalArguments = clark2dt::AlmostInfiniteClark2DtDispersalArguments;
#[cfg(not(feature = "almost-infinite-clark2dt-dispersal"))]
type Clark2DtDispersalArguments = !;

impl AlmostInfiniteArguments {
    #[allow(clippy::missing_errors_doc)]
    pub fn try_load(
        self,
    ) -> Result<Either<NormalDispersalArguments, Clark2DtDispersalArguments>, String> {
        match self {
            #[cfg(feature = "almost-infinite-normal-dispersal")]
            Self {
                sample: Sample::Circle { centre, radius },
                dispersal: Dispersal::Normal { sigma },
            } => Ok(Either::Left(
                normal::AlmostInfiniteNormalDispersalArguments {
                    centre,
                    radius,
                    sigma,
                },
            )),
            #[cfg(feature = "almost-infinite-normal-dispersal")]
            Self {
                sample,
                dispersal: Dispersal::Normal { .. },
            } => Err(format!(
                "Normal dispersal does not yet support {sample:?} sampling"
            )),
            #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
            Self {
                sample: Sample::Rectangle(sample),
                dispersal: Dispersal::Clark2Dt { shape_u, tail_p },
            } => Ok(Either::Right(
                clark2dt::AlmostInfiniteClark2DtDispersalArguments {
                    sample,
                    shape_u,
                    tail_p,
                },
            )),
            #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
            Self {
                sample,
                dispersal: Dispersal::Clark2Dt { .. },
            } => Err(format!(
                "Clark2Dt dispersal does not yet support {sample:?} sampling"
            )),
        }
    }

    #[cfg(feature = "almost-infinite-normal-dispersal")]
    #[must_use]
    pub fn from_normal(args: &normal::AlmostInfiniteNormalDispersalArguments) -> Self {
        Self {
            sample: Sample::Circle {
                centre: args.centre.clone(),
                radius: args.radius,
            },
            dispersal: Dispersal::Normal { sigma: args.sigma },
        }
    }

    #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
    #[must_use]
    pub fn from_clark2dt(args: &clark2dt::AlmostInfiniteClark2DtDispersalArguments) -> Self {
        Self {
            sample: Sample::Rectangle(args.sample.clone()),
            dispersal: Dispersal::Clark2Dt {
                shape_u: args.shape_u,
                tail_p: args.tail_p,
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Sample {
    Circle {
        #[serde(default = "default_circle_sample_centre")]
        centre: Location,
        radius: u16,
    },
    Rectangle(LandscapeExtent),
}

#[must_use]
pub const fn default_circle_sample_centre() -> Location {
    const HABITAT_CENTRE: u32 = u32::MAX / 2;

    Location::new(HABITAT_CENTRE, HABITAT_CENTRE)
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
enum Dispersal {
    #[cfg(feature = "almost-infinite-normal-dispersal")]
    #[serde(alias = "Gaussian")]
    Normal { sigma: NonNegativeF64 },
    #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
    Clark2Dt {
        #[serde(alias = "u")]
        shape_u: PositiveF64,
        #[serde(default = "PositiveF64::one")]
        #[serde(alias = "p")]
        tail_p: PositiveF64,
    },
}

use either::Either;
use necsim_impls_no_std::cogs::{
    habitat::almost_infinite::AlmostInfiniteHabitat,
    origin_sampler::{
        almost_infinite::{
            circle::AlmostInfiniteCircleOriginSampler,
            rectangle::AlmostInfiniteRectangleOriginSampler, AlmostInfiniteOriginSampler,
        },
        pre_sampler::OriginPreSampler,
    },
};
use serde::{Deserialize, Serialize};

use necsim_core::{
    cogs::MathsCore,
    landscape::{LandscapeExtent, Location},
};
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
    #[must_use]
    pub fn load(self) -> Either<NormalDispersalArguments, Clark2DtDispersalArguments> {
        match self {
            #[cfg(feature = "almost-infinite-normal-dispersal")]
            Self {
                sample,
                dispersal: Dispersal::Normal { sigma },
            } => Either::Left(normal::AlmostInfiniteNormalDispersalArguments { sample, sigma }),
            #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
            Self {
                sample,
                dispersal: Dispersal::Clark2Dt { shape_u, tail_p },
            } => Either::Right(clark2dt::AlmostInfiniteClark2DtDispersalArguments {
                sample,
                shape_u,
                tail_p,
            }),
        }
    }

    #[cfg(feature = "almost-infinite-normal-dispersal")]
    #[must_use]
    pub fn from_normal(args: &normal::AlmostInfiniteNormalDispersalArguments) -> Self {
        Self {
            sample: args.sample.clone(),
            dispersal: Dispersal::Normal { sigma: args.sigma },
        }
    }

    #[cfg(feature = "almost-infinite-clark2dt-dispersal")]
    #[must_use]
    pub fn from_clark2dt(args: &clark2dt::AlmostInfiniteClark2DtDispersalArguments) -> Self {
        Self {
            sample: args.sample.clone(),
            dispersal: Dispersal::Clark2Dt {
                shape_u: args.shape_u,
                tail_p: args.tail_p,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Sample {
    Circle {
        #[serde(default = "Sample::default_circle_sample_centre")]
        centre: Location,
        radius: u16,
    },
    Rectangle(LandscapeExtent),
}

impl Sample {
    #[must_use]
    pub const fn default_circle_sample_centre() -> Location {
        const HABITAT_CENTRE: u32 = u32::MAX / 2;

        Location::new(HABITAT_CENTRE, HABITAT_CENTRE)
    }

    pub fn into_origin_sampler<M: MathsCore, I: Iterator<Item = u64>>(
        self,
        habitat: &AlmostInfiniteHabitat<M>,
        pre_sampler: OriginPreSampler<M, I>,
    ) -> AlmostInfiniteOriginSampler<M, I> {
        match self {
            Self::Circle { centre, radius } => AlmostInfiniteOriginSampler::Circle(
                AlmostInfiniteCircleOriginSampler::new(pre_sampler, habitat, centre, radius),
            ),
            Self::Rectangle(sample) => AlmostInfiniteOriginSampler::Rectangle(
                AlmostInfiniteRectangleOriginSampler::new(pre_sampler, habitat, sample),
            ),
        }
    }
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

use serde::{Serialize, Serializer};

use necsim_partitioning_core::partition::Partition;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
use rustcoalescence_algorithms::AlgorithmParamters;

#[derive(Debug, DeserializeState)]
#[serde(deserialize_state = "Partition")]
pub enum Algorithm {
    #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
    #[serde(alias = "Classical")]
    Gillespie(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::gillespie::GillespieAlgorithm as rustcoalescence_algorithms::AlgorithmParamters>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
    #[serde(alias = "SkippingGillespie")]
    EventSkipping(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::event_skipping::EventSkippingAlgorithm as AlgorithmParamters>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
    #[serde(alias = "CUDA")]
    Cuda(#[serde(deserialize_state)] <rustcoalescence_algorithms_cuda::CudaAlgorithm as AlgorithmParamters>::Arguments),
    #[cfg(feature = "rustcoalescence-algorithms-independent")]
    Independent(
        #[serde(deserialize_state)] <rustcoalescence_algorithms_independent::IndependentAlgorithm as AlgorithmParamters>::Arguments,
    ),
}

impl Serialize for Algorithm {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[allow(unreachable_patterns, clippy::single_match_else)]
        match self {
            #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
            Self::Gillespie(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 0, "Gillespie", args)
            },
            #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
            Self::EventSkipping(args) => serializer.serialize_newtype_variant(
                stringify!(Algorithm),
                1,
                "EventSkipping",
                args,
            ),
            #[cfg(feature = "rustcoalescence-algorithms-cuda")]
            Self::Cuda(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 2, "CUDA", args)
            },
            #[cfg(feature = "rustcoalescence-algorithms-independent")]
            Self::Independent(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 3, "Independent", args)
            },
            _ => {
                std::mem::drop(serializer);

                Err(serde::ser::Error::custom(
                    "rustcoalescence must be compiled to support at least one algorithm",
                ))
            },
        }
    }
}

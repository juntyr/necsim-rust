use rustcoalescence_algorithms::AlgorithmParamters;

use crate::arguments::MonolithicArguments;

mod classical;
mod turnover;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum GillespieAlgorithm {}

impl AlgorithmParamters for GillespieAlgorithm {
    type Arguments = MonolithicArguments;
    type Error = !;
}

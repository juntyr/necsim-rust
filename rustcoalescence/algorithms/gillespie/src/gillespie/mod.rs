use necsim_core::cogs::MathsCore;
use necsim_impls_no_std::cogs::maths::intrinsics::IntrinsicsMathsCore;
use necsim_impls_std::cogs::rng::pcg::Pcg;

use rustcoalescence_algorithms::{AlgorithmDefaults, AlgorithmParamters};

use crate::arguments::GillespieArguments;

mod classical;
mod turnover;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub enum GillespieAlgorithm {}

impl AlgorithmParamters for GillespieAlgorithm {
    type Arguments = GillespieArguments;
    type Error = !;
}

impl AlgorithmDefaults for GillespieAlgorithm {
    type MathsCore = IntrinsicsMathsCore;
    type Rng<M: MathsCore> = Pcg<M>;
}

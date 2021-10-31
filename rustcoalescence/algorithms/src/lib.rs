#![deny(clippy::pedantic)]

use necsim_core::{
    cogs::{LineageReference, LineageStore, MathsCore, RngCore},
    reporter::Reporter,
};
use necsim_core_bond::NonNegativeF64;

use necsim_impls_no_std::cogs::origin_sampler::pre_sampler::OriginPreSampler;
use necsim_partitioning_core::LocalPartition;

use rustcoalescence_scenarios::Scenario;

pub trait AlgorithmArguments {
    type Arguments;
}

pub trait Algorithm<O: Scenario<Self::MathsCore, Self::Rng>, R: Reporter, P: LocalPartition<R>>:
    Sized + AlgorithmArguments
{
    type Error;

    type MathsCore: MathsCore;
    type Rng: RngCore<Self::MathsCore>;
    type LineageReference: LineageReference<Self::MathsCore, O::Habitat>;
    type LineageStore: LineageStore<Self::MathsCore, O::Habitat, Self::LineageReference>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising or running the algorithm failed
    fn initialise_and_simulate<I: Iterator<Item = u64>>(
        args: Self::Arguments,
        rng: Self::Rng,
        scenario: O,
        pre_sampler: OriginPreSampler<Self::MathsCore, I>,
        pause_before: Option<NonNegativeF64>,
        local_partition: &mut P,
    ) -> Result<(NonNegativeF64, u64), Self::Error>;
}

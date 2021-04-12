#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(never_type)]

#[macro_use]
extern crate log;

use std::num::NonZeroU32;

use necsim_core::cogs::{
    DispersalSampler, Habitat, LineageReference, LineageStore, OriginSampler, RngCore,
    SpeciationProbability, TurnoverRate,
};

use necsim_impls_no_std::{
    cogs::origin_sampler::pre_sampler::OriginPreSampler, decomposition::Decomposition,
};

use necsim_impls_std::bounded::ZeroExclOneInclF64;

pub mod almost_infinite;
pub mod non_spatial;
pub mod spatially_explicit;
pub mod spatially_implicit;

pub trait ScenarioArguments {
    type Arguments;
}

pub trait Scenario<G: RngCore, L: LineageStore<Self::Habitat, Self::LineageReference>>:
    Sized + ScenarioArguments
{
    type Error;

    type Habitat: Habitat;
    type OriginSampler<'h, I: Iterator<Item = u64>>: OriginSampler<'h, Habitat = Self::Habitat>;
    type Decomposition: Decomposition<Self::Habitat>;
    type LineageReference: LineageReference<Self::Habitat>;
    type LineageStore: LineageStore<Self::Habitat, Self::LineageReference>;
    type DispersalSampler: DispersalSampler<Self::Habitat, G>;
    type TurnoverRate: TurnoverRate<Self::Habitat>;
    type SpeciationProbability: SpeciationProbability<Self::Habitat>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the scenario failed
    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error>;

    fn build(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    );

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<I>,
    ) -> Self::OriginSampler<'_, I>;

    fn decompose(&self, rank: u32, partitions: NonZeroU32) -> Self::Decomposition;

    fn habitat(&self) -> &Self::Habitat;
}

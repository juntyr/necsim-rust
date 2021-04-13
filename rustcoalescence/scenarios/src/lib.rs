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
    bounded::ZeroExclOneInclF64,
    cogs::{
        dispersal_sampler::in_memory::InMemoryDispersalSampler,
        origin_sampler::pre_sampler::OriginPreSampler,
    },
    decomposition::Decomposition,
};

pub mod almost_infinite;
pub mod non_spatial;
pub mod spatially_explicit;
pub mod spatially_implicit;

pub trait ScenarioArguments {
    type Arguments;
}

pub trait Scenario<G: RngCore>: Sized + ScenarioArguments {
    type Error;

    type Habitat: Habitat;
    type OriginSampler<'h, I: Iterator<Item = u64>>: OriginSampler<'h, Habitat = Self::Habitat>;
    type Decomposition: Decomposition<Self::Habitat>;
    type LineageReference: LineageReference<Self::Habitat>;
    type LineageStore<L: LineageStore<Self::Habitat, Self::LineageReference>>: LineageStore<
        Self::Habitat,
        Self::LineageReference,
    >;
    type DispersalSampler<D: DispersalSampler<Self::Habitat, G>>: DispersalSampler<Self::Habitat, G>;
    type TurnoverRate: TurnoverRate<Self::Habitat>;
    type SpeciationProbability: SpeciationProbability<Self::Habitat>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the scenario failed
    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error>;

    /// Inside rustcoalescence, I know that only specialised
    /// `InMemoryDispersalSampler` implementations will be requested.
    fn build<D: InMemoryDispersalSampler<Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    );

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<I>,
    ) -> Self::OriginSampler<'_, I>;

    fn habitat(&self) -> &Self::Habitat;

    fn decompose(habitat: &Self::Habitat, rank: u32, partitions: NonZeroU32)
        -> Self::Decomposition;
}

#![deny(clippy::pedantic)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]
#![feature(never_type)]

#[macro_use]
extern crate log;

use std::num::NonZeroU32;

use necsim_core::cogs::{
    DispersalSampler, F64Core, Habitat, LineageReference, LineageStore, OriginSampler, RngCore,
    SpeciationProbability, TurnoverRate,
};
use necsim_core_bond::PositiveUnitF64;

use necsim_impls_no_std::{
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

pub trait Scenario<F: F64Core, G: RngCore<F>>: Sized + ScenarioArguments {
    type Error;

    type Habitat: Habitat<F>;
    type OriginSampler<'h, I: Iterator<Item = u64>>: OriginSampler<'h, F, Habitat = Self::Habitat>;
    type Decomposition: Decomposition<F, Self::Habitat>;
    type LineageReference: LineageReference<F, Self::Habitat>;
    type LineageStore<L: LineageStore<F, Self::Habitat, Self::LineageReference>>: LineageStore<
        F,
        Self::Habitat,
        Self::LineageReference,
    >;
    type DispersalSampler<D: DispersalSampler<F, Self::Habitat, G>>: DispersalSampler<
        F,
        Self::Habitat,
        G,
    >;
    type TurnoverRate: TurnoverRate<F, Self::Habitat>;
    type SpeciationProbability: SpeciationProbability<F, Self::Habitat>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the scenario failed
    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error>;

    /// Inside rustcoalescence, I know that only specialised
    /// `InMemoryDispersalSampler` implementations will be requested.
    fn build<D: InMemoryDispersalSampler<F, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    );

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<F, I>,
    ) -> Self::OriginSampler<'_, I>;

    fn habitat(&self) -> &Self::Habitat;

    fn decompose(habitat: &Self::Habitat, rank: u32, partitions: NonZeroU32)
        -> Self::Decomposition;
}

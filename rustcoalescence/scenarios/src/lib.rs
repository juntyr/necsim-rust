#![deny(clippy::pedantic)]
#![feature(associated_type_bounds)]
#![feature(never_type)]
#![allow(incomplete_features)]
#![feature(generic_associated_types)]

#[macro_use]
extern crate log;

use necsim_core::cogs::{
    DispersalSampler, Habitat, LineageReference, LineageStore, MathsCore, OriginSampler, RngCore,
    SpeciationProbability, TurnoverRate,
};
use necsim_core_bond::{Partition, PositiveUnitF64};

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

pub trait ScenarioParameters {
    type Arguments;
    type Error;
}

pub trait Scenario<M: MathsCore, G: RngCore<M>>: Sized + ScenarioParameters {
    type Habitat: Habitat<M>;
    type OriginSampler<'h, I: Iterator<Item = u64>>: OriginSampler<'h, M, Habitat = Self::Habitat>;
    type Decomposition: Decomposition<M, Self::Habitat>;
    type LineageReference: LineageReference<M, Self::Habitat>;
    type LineageStore<L: LineageStore<M, Self::Habitat, Self::LineageReference>>: LineageStore<
        M,
        Self::Habitat,
        Self::LineageReference,
    >;
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>>: DispersalSampler<
        M,
        Self::Habitat,
        G,
    >;
    type TurnoverRate: TurnoverRate<M, Self::Habitat>;
    type SpeciationProbability: SpeciationProbability<M, Self::Habitat>;

    /// # Errors
    ///
    /// Returns a `Self::Error` if initialising the scenario failed
    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error>;

    /// Inside rustcoalescence, I know that only specialised
    /// `InMemoryDispersalSampler` implementations will be requested.
    fn build<D: InMemoryDispersalSampler<M, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    );

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<M, I>,
    ) -> Self::OriginSampler<'_, I>;

    fn habitat(&self) -> &Self::Habitat;

    fn decompose(habitat: &Self::Habitat, subdomain: Partition) -> Self::Decomposition;
}

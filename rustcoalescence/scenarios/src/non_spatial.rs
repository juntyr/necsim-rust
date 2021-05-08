use std::num::NonZeroU32;

use serde::Deserialize;

use necsim_core::cogs::{DispersalSampler, LineageStore, RngCore};
use necsim_core_bond::ZeroExclOneInclF64;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
        habitat::non_spatial::NonSpatialHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::modulo::ModuloDecomposition,
};

use crate::{Scenario, ScenarioArguments};

#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialScenario<G: RngCore> {
    habitat: NonSpatialHabitat,
    dispersal_sampler: NonSpatialDispersalSampler<G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "NonSpatial")]
pub struct NonSpatialArguments {
    pub area: (u32, u32),
    pub deme: u32,
}

impl<G: RngCore> ScenarioArguments for NonSpatialScenario<G> {
    type Arguments = NonSpatialArguments;
}

impl<G: RngCore> Scenario<G> for NonSpatialScenario<G> {
    type Decomposition = ModuloDecomposition;
    type DispersalSampler<D: DispersalSampler<Self::Habitat, G>> = NonSpatialDispersalSampler<G>;
    type Error = !;
    type Habitat = NonSpatialHabitat;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = NonSpatialOriginSampler<'h, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error> {
        let habitat = NonSpatialHabitat::new(args.area, args.deme);
        let dispersal_sampler = NonSpatialDispersalSampler::default();
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        Ok(Self {
            habitat,
            dispersal_sampler,
            turnover_rate,
            speciation_probability,
        })
    }

    fn build<D: DispersalSampler<Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    ) {
        (
            self.habitat,
            self.dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
        )
    }

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<I>,
    ) -> Self::OriginSampler<'_, I> {
        NonSpatialOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        rank: u32,
        partitions: NonZeroU32,
    ) -> Self::Decomposition {
        ModuloDecomposition::new(rank, partitions)
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

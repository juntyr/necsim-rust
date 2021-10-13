use std::num::NonZeroU32;

use serde::Deserialize;

use necsim_core::cogs::{DispersalSampler, F64Core, LineageStore, RngCore};
use necsim_core_bond::PositiveUnitF64;

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
pub struct NonSpatialScenario<F: F64Core, G: RngCore<F>> {
    habitat: NonSpatialHabitat<F>,
    dispersal_sampler: NonSpatialDispersalSampler<F, G>,
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

impl<F: F64Core, G: RngCore<F>> ScenarioArguments for NonSpatialScenario<F, G> {
    type Arguments = NonSpatialArguments;
}

impl<F: F64Core, G: RngCore<F>> Scenario<F, G> for NonSpatialScenario<F, G> {
    type Decomposition = ModuloDecomposition;
    type DispersalSampler<D: DispersalSampler<F, Self::Habitat, G>> =
        NonSpatialDispersalSampler<F, G>;
    type Error = !;
    type Habitat = NonSpatialHabitat<F>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<F, Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = NonSpatialOriginSampler<'h, F, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
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

    fn build<D: DispersalSampler<F, Self::Habitat, G>>(
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
        pre_sampler: OriginPreSampler<F, I>,
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

use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use necsim_core::cogs::{DispersalSampler, LineageStore, MathsCore, RngCore};
use necsim_core_bond::{Partition, PositiveUnitF64};

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
pub struct NonSpatialScenario<M: MathsCore, G: RngCore<M>> {
    habitat: NonSpatialHabitat<M>,
    dispersal_sampler: NonSpatialDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialArguments {
    pub area: (u32, u32),
    pub deme: NonZeroU32,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioArguments for NonSpatialScenario<M, G> {
    type Arguments = NonSpatialArguments;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G> for NonSpatialScenario<M, G> {
    type Decomposition = ModuloDecomposition;
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        NonSpatialDispersalSampler<M, G>;
    type Error = !;
    type Habitat = NonSpatialHabitat<M>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<M, Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = NonSpatialOriginSampler<'h, M, I>;
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

    fn build<D: DispersalSampler<M, Self::Habitat, G>>(
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
        pre_sampler: OriginPreSampler<M, I>,
    ) -> Self::OriginSampler<'_, I> {
        NonSpatialOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(_habitat: &Self::Habitat, subdomain: Partition) -> Self::Decomposition {
        ModuloDecomposition::new(subdomain)
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

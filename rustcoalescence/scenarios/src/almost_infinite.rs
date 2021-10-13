use std::num::NonZeroU32;

use serde::Deserialize;

use necsim_core::cogs::{DispersalSampler, F64Core, LineageStore, RngCore};
use necsim_core_bond::{NonNegativeF64, PositiveUnitF64};

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_normal::AlmostInfiniteNormalDispersalSampler,
        habitat::almost_infinite::AlmostInfiniteHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        lineage_store::coherent::globally::almost_infinite::AlmostInfiniteLineageStore,
        origin_sampler::{
            almost_infinite::AlmostInfiniteOriginSampler, pre_sampler::OriginPreSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::radial::RadialDecomposition,
};

use crate::{Scenario, ScenarioArguments};

#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteScenario<F: F64Core, G: RngCore<F>> {
    radius: u32,

    habitat: AlmostInfiniteHabitat<F>,
    dispersal_sampler: AlmostInfiniteNormalDispersalSampler<F, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "AlmostInfinite")]
pub struct AlmostInfiniteArguments {
    pub radius: u32,
    pub sigma: NonNegativeF64,
}

impl<F: F64Core, G: RngCore<F>> ScenarioArguments for AlmostInfiniteScenario<F, G> {
    type Arguments = AlmostInfiniteArguments;
}

impl<F: F64Core, G: RngCore<F>> Scenario<F, G> for AlmostInfiniteScenario<F, G> {
    type Decomposition = RadialDecomposition;
    type DispersalSampler<D: DispersalSampler<F, Self::Habitat, G>> =
        AlmostInfiniteNormalDispersalSampler<F, G>;
    type Error = !;
    type Habitat = AlmostInfiniteHabitat<F>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<F, Self::Habitat, Self::LineageReference>> =
        AlmostInfiniteLineageStore<F>;
    type OriginSampler<'h, I: Iterator<Item = u64>> = AlmostInfiniteOriginSampler<'h, F, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(args.sigma);
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        Ok(Self {
            radius: args.radius,

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
        AlmostInfiniteOriginSampler::new(pre_sampler, &self.habitat, self.radius)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        rank: u32,
        partitions: NonZeroU32,
    ) -> Self::Decomposition {
        RadialDecomposition::new(rank, partitions)
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

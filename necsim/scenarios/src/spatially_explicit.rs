use std::num::NonZeroU32;

use array2d::Array2D;
use necsim_core::cogs::{Habitat, LineageStore, RngCore};
use thiserror::Error;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::in_memory::packed_alias::InMemoryPackedAliasDispersalSampler,
        habitat::in_memory::InMemoryHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::equal_area::EqualAreaDecomposition,
};

use necsim_impls_std::{
    bounded::ZeroExclOneInclF64,
    cogs::dispersal_sampler::in_memory::{
        error::InMemoryDispersalSamplerError, InMemoryDispersalSampler,
    },
};

use crate::{Scenario, ScenarioArguments};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitScenario<G: RngCore> {
    habitat: InMemoryHabitat,
    dispersal_sampler: InMemoryPackedAliasDispersalSampler<InMemoryHabitat, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryArguments {
    pub habitat_map: Array2D<u32>,
    pub dispersal_map: Array2D<f64>,
}

#[derive(Debug, Error)]
#[error("{0} is negative.")]
#[allow(clippy::module_name_repetitions)]
pub struct NonNegativeF64Error(f64);

impl<G: RngCore> ScenarioArguments for SpatiallyExplicitScenario<G> {
    type Arguments = InMemoryArguments;
}

impl<G: RngCore, L: LineageStore<InMemoryHabitat, InMemoryLineageReference>> Scenario<G, L>
    for SpatiallyExplicitScenario<G>
{
    type Decomposition = EqualAreaDecomposition<Self::Habitat>;
    type DispersalSampler = InMemoryPackedAliasDispersalSampler<Self::Habitat, G>;
    type Error = InMemoryDispersalSamplerError;
    type Habitat = InMemoryHabitat;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = InMemoryOriginSampler<'h, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error> {
        let habitat = InMemoryHabitat::new(args.habitat_map);
        let dispersal_sampler =
            InMemoryPackedAliasDispersalSampler::new(&args.dispersal_map, &habitat)?;
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.get());

        Ok(Self {
            habitat,
            dispersal_sampler,
            turnover_rate,
            speciation_probability,
        })
    }

    fn build(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler,
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
        InMemoryOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(&self, rank: u32, partitions: NonZeroU32) -> Self::Decomposition {
        match EqualAreaDecomposition::new(&self.habitat, rank, partitions) {
            Ok(decomposition) => decomposition,
            Err(decomposition) => {
                warn!(
                    "Spatially explicit habitat of size {}x{} could not be partitioned into {} \
                     partition(s).",
                    self.habitat.get_extent().width(),
                    self.habitat.get_extent().height(),
                    partitions.get(),
                );

                decomposition
            },
        }
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

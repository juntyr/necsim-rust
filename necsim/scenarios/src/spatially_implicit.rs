use std::num::NonZeroU32;

use serde::Deserialize;

use necsim_core::cogs::{LineageStore, RngCore};

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::spatially_implicit::SpatiallyImplicitDispersalSampler,
        habitat::spatially_implicit::SpatiallyImplicitHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{
            pre_sampler::OriginPreSampler, spatially_implicit::SpatiallyImplicitOriginSampler,
        },
        speciation_probability::spatially_implicit::SpatiallyImplicitSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::modulo::ModuloDecomposition,
};

use necsim_impls_std::bounded::ZeroExclOneInclF64;

use crate::{Scenario, ScenarioArguments};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitScenario<G: RngCore> {
    habitat: SpatiallyImplicitHabitat,
    dispersal_sampler: SpatiallyImplicitDispersalSampler<G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: SpatiallyImplicitSpeciationProbability,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "SpatiallyImplicit")]
pub struct SpatiallyImplicitArguments {
    pub local_area: (u32, u32),
    pub local_deme: u32,
    pub meta_area: (u32, u32),
    pub meta_deme: u32,

    #[serde(alias = "migration")]
    pub migration_probability_per_generation: ZeroExclOneInclF64,
}

impl<G: RngCore> ScenarioArguments for SpatiallyImplicitScenario<G> {
    type Arguments = SpatiallyImplicitArguments;
}

impl<G: RngCore, L: LineageStore<SpatiallyImplicitHabitat, InMemoryLineageReference>> Scenario<G, L>
    for SpatiallyImplicitScenario<G>
{
    type Decomposition = ModuloDecomposition;
    type DispersalSampler = SpatiallyImplicitDispersalSampler<G>;
    type Error = !;
    type Habitat = SpatiallyImplicitHabitat;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = SpatiallyImplicitOriginSampler<'h, I>;
    type SpeciationProbability = SpatiallyImplicitSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error> {
        let habitat = SpatiallyImplicitHabitat::new(
            args.local_area,
            args.local_deme,
            args.meta_area,
            args.meta_deme,
        );
        let dispersal_sampler =
            SpatiallyImplicitDispersalSampler::new(args.migration_probability_per_generation.get());
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability = SpatiallyImplicitSpeciationProbability::new(
            speciation_probability_per_generation.get(),
        );

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
        SpatiallyImplicitOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(&self, rank: u32, partitions: NonZeroU32) -> Self::Decomposition {
        ModuloDecomposition::new(rank, partitions)
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

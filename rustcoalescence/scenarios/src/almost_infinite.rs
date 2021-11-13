use serde::{Deserialize, Serialize};

use necsim_core::cogs::{DispersalSampler, LineageStore, MathsCore, RngCore};
use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::partition::Partition;

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

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteScenario<M: MathsCore, G: RngCore<M>> {
    radius: u32,

    habitat: AlmostInfiniteHabitat<M>,
    dispersal_sampler: AlmostInfiniteNormalDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteArguments {
    pub radius: u32,
    pub sigma: NonNegativeF64,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioParameters for AlmostInfiniteScenario<M, G> {
    type Arguments = AlmostInfiniteArguments;
    type Error = !;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G> for AlmostInfiniteScenario<M, G> {
    type Decomposition = RadialDecomposition;
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        AlmostInfiniteNormalDispersalSampler<M, G>;
    type Habitat = AlmostInfiniteHabitat<M>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<M, Self::Habitat, Self::LineageReference>> =
        AlmostInfiniteLineageStore<M>;
    type OriginSampler<'h, I: Iterator<Item = u64>> = AlmostInfiniteOriginSampler<'h, M, I>;
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
        AlmostInfiniteOriginSampler::new(pre_sampler, &self.habitat, self.radius)
    }

    fn decompose(_habitat: &Self::Habitat, subdomain: Partition) -> Self::Decomposition {
        RadialDecomposition::new(subdomain)
    }

    fn habitat(&self) -> &Self::Habitat {
        &self.habitat
    }
}

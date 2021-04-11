use core::num::NonZeroU32;

use necsim_core::cogs::{LineageStore, RngCore};

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

use crate::{
    bounded::{NonNegativeF64, ZeroExclOneInclF64},
    scenario::Scenario,
};

#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteScenario<G: RngCore> {
    radius: u32,

    habitat: AlmostInfiniteHabitat,
    dispersal_sampler: AlmostInfiniteNormalDispersalSampler<G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteArguments {
    pub radius: u32,
    pub sigma: NonNegativeF64,
}

impl<G: RngCore, L: LineageStore<AlmostInfiniteHabitat, InMemoryLineageReference>> Scenario<G, L>
    for AlmostInfiniteScenario<G>
{
    type Arguments = AlmostInfiniteArguments;
    type Decomposition = RadialDecomposition;
    type DispersalSampler = AlmostInfiniteNormalDispersalSampler<G>;
    type Error = !;
    type Habitat = AlmostInfiniteHabitat;
    type LineageReference = InMemoryLineageReference;
    type LineageStore = AlmostInfiniteLineageStore;
    type OriginSampler<'h, I: Iterator<Item = u64>> = AlmostInfiniteOriginSampler<'h, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: ZeroExclOneInclF64,
    ) -> Result<Self, Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteNormalDispersalSampler::new(args.sigma.get());
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.get());

        Ok(Self {
            radius: args.radius,

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
        AlmostInfiniteOriginSampler::new(pre_sampler, &self.habitat, self.radius)
    }

    fn decompose(&self, rank: u32, partitions: NonZeroU32) -> Self::Decomposition {
        RadialDecomposition::new(rank, partitions)
    }
}

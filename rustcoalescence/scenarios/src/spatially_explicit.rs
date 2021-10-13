use std::{marker::PhantomData, num::NonZeroU32};

use thiserror::Error;

use necsim_core::cogs::{DispersalSampler, F64Core, Habitat, LineageStore, RngCore};
use necsim_core_bond::PositiveUnitF64;

use necsim_impls_no_std::{
    array2d::Array2D,
    cogs::{
        dispersal_sampler::in_memory::{
            contract::explicit_in_memory_dispersal_check_contract, InMemoryDispersalSampler,
        },
        habitat::in_memory::InMemoryHabitat,
        lineage_reference::in_memory::InMemoryLineageReference,
        origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::equal::EqualDecomposition,
};

use necsim_impls_std::cogs::dispersal_sampler::in_memory::error::InMemoryDispersalSamplerError;

use crate::{Scenario, ScenarioArguments};

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitScenario<F: F64Core, G: RngCore<F>> {
    habitat: InMemoryHabitat<F>,
    dispersal_map: Array2D<f64>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
    _marker: PhantomData<G>,
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

impl<F: F64Core, G: RngCore<F>> ScenarioArguments for SpatiallyExplicitScenario<F, G> {
    type Arguments = InMemoryArguments;
}

impl<F: F64Core, G: RngCore<F>> Scenario<F, G> for SpatiallyExplicitScenario<F, G> {
    type Decomposition = EqualDecomposition<F, Self::Habitat>;
    type DispersalSampler<D: DispersalSampler<F, Self::Habitat, G>> = D;
    type Error = InMemoryDispersalSamplerError;
    type Habitat = InMemoryHabitat<F>;
    type LineageReference = InMemoryLineageReference;
    type LineageStore<L: LineageStore<F, Self::Habitat, Self::LineageReference>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = InMemoryOriginSampler<'h, F, I>;
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = InMemoryHabitat::new(args.habitat_map);
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        let habitat_extent = habitat.get_extent();
        let habitat_area = (habitat_extent.width() as usize) * (habitat_extent.height() as usize);

        if args.dispersal_map.num_rows() != habitat_area
            || args.dispersal_map.num_columns() != habitat_area
        {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalMapSize);
        }

        if !explicit_in_memory_dispersal_check_contract(&args.dispersal_map, &habitat) {
            return Err(InMemoryDispersalSamplerError::InconsistentDispersalProbabilities);
        }

        Ok(Self {
            habitat,
            dispersal_map: args.dispersal_map,
            turnover_rate,
            speciation_probability,
            _marker: PhantomData::<G>,
        })
    }

    fn build<D: InMemoryDispersalSampler<F, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
    ) {
        let dispersal_sampler = D::unchecked_new(&self.dispersal_map, &self.habitat);

        (
            self.habitat,
            dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
        )
    }

    fn sample_habitat<I: Iterator<Item = u64>>(
        &self,
        pre_sampler: OriginPreSampler<F, I>,
    ) -> Self::OriginSampler<'_, I> {
        InMemoryOriginSampler::new(pre_sampler, &self.habitat)
    }

    fn decompose(
        habitat: &Self::Habitat,
        rank: u32,
        partitions: NonZeroU32,
    ) -> Self::Decomposition {
        match EqualDecomposition::weight(habitat, rank, partitions) {
            Ok(decomposition) => decomposition,
            Err(decomposition) => {
                warn!(
                    "Spatially explicit habitat of size {}x{} could not be partitioned into {} \
                     partition(s).",
                    habitat.get_extent().width(),
                    habitat.get_extent().height(),
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

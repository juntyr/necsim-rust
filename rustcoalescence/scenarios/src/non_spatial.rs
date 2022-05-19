use std::num::NonZeroU32;

use serde::{Deserialize, Serialize};

use necsim_core::cogs::{
    distribution::IndexU64, DispersalSampler, LineageStore, MathsCore, Rng, Samples,
};
use necsim_core_bond::{OffByOneU32, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::non_spatial::NonSpatialDispersalSampler,
        habitat::non_spatial::NonSpatialHabitat,
        origin_sampler::{non_spatial::NonSpatialOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::modulo::ModuloDecomposition,
};

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialScenario<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> {
    habitat: NonSpatialHabitat<M>,
    dispersal_sampler: NonSpatialDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialArguments {
    pub area: (OffByOneU32, OffByOneU32),
    pub deme: NonZeroU32,
}

impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> ScenarioParameters
    for NonSpatialScenario<M, G>
{
    type Arguments = NonSpatialArguments;
    type Error = !;
}

impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> Scenario<M, G> for NonSpatialScenario<M, G> {
    type Decomposition = ModuloDecomposition;
    type DecompositionAuxiliary = ();
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        NonSpatialDispersalSampler<M, G>;
    type Habitat = NonSpatialHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = NonSpatialOriginSampler<'h, M, I> where G: 'h;
    type OriginSamplerAuxiliary = ();
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
        Self::OriginSamplerAuxiliary,
        Self::DecompositionAuxiliary,
    ) {
        (
            self.habitat,
            self.dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
            (),
            (),
        )
    }

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<M, I>,
        _auxiliary: Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h,
    {
        NonSpatialOriginSampler::new(pre_sampler, habitat)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        ModuloDecomposition::new(subdomain)
    }
}

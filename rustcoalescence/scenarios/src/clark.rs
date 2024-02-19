use serde::{Deserialize, Serialize};

use necsim_core::{
    cogs::{DispersalSampler, LineageStore, MathsCore, RngCore},
    landscape::LandscapeExtent,
};
use necsim_core_bond::{OpenClosedUnitF64 as PositiveUnitF64, PositiveF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_clark::AlmostInfiniteClarkDispersalSampler,
        habitat::almost_infinite::AlmostInfiniteHabitat,
        lineage_store::coherent::globally::singleton_demes::SingletonDemesLineageStore,
        origin_sampler::{clark::ClarkOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::radial::RadialDecomposition,
};

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
pub struct ClarkScenario<M: MathsCore, G: RngCore<M>> {
    sample: LandscapeExtent,

    habitat: AlmostInfiniteHabitat<M>,
    dispersal_sampler: AlmostInfiniteClarkDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "Clark")]
pub struct ClarkArguments {
    pub sample: LandscapeExtent,
    pub shape: PositiveF64,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioParameters for ClarkScenario<M, G> {
    type Arguments = ClarkArguments;
    type Error = !;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G> for ClarkScenario<M, G> {
    type Decomposition = RadialDecomposition;
    type DecompositionAuxiliary = ();
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        AlmostInfiniteClarkDispersalSampler<M, G>;
    type Habitat = AlmostInfiniteHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> =
        SingletonDemesLineageStore<M, Self::Habitat>;
    type OriginSampler<'h, I: Iterator<Item = u64>> = ClarkOriginSampler<'h, M, I> where G: 'h;
    type OriginSamplerAuxiliary = (LandscapeExtent,);
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler = AlmostInfiniteClarkDispersalSampler::new(args.shape);
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        Ok(Self {
            sample: args.sample,

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
            (self.sample,),
            (),
        )
    }

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<M, I>,
        (sample,): Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h,
    {
        ClarkOriginSampler::new(pre_sampler, habitat, sample)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        RadialDecomposition::new(subdomain)
    }
}

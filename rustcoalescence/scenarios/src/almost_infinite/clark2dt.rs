use serde::{Deserialize, Serialize};

use necsim_core::{
    cogs::{DispersalSampler, LineageStore, MathsCore, RngCore},
    landscape::LandscapeExtent,
};
use necsim_core_bond::{OpenClosedUnitF64 as PositiveUnitF64, PositiveF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_clark2dt::AlmostInfiniteClark2DtDispersalSampler,
        habitat::almost_infinite::AlmostInfiniteHabitat,
        lineage_store::coherent::globally::singleton_demes::SingletonDemesLineageStore,
        origin_sampler::{
            almost_infinite_rectangle::AlmostInfiniteRectangleOriginSampler,
            pre_sampler::OriginPreSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::radial::RadialDecomposition,
};

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteClark2DtDispersalScenario<M: MathsCore, G: RngCore<M>> {
    sample: LandscapeExtent,

    habitat: AlmostInfiniteHabitat<M>,
    dispersal_sampler: AlmostInfiniteClark2DtDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "AlmostInfiniteClark2DtDispersal")]
pub struct AlmostInfiniteClark2DtDispersalArguments {
    pub sample: LandscapeExtent,
    #[serde(alias = "u")]
    pub shape_u: PositiveF64,
    #[serde(default = "PositiveF64::one")]
    #[serde(alias = "p")]
    pub tail_p: PositiveF64,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioParameters
    for AlmostInfiniteClark2DtDispersalScenario<M, G>
{
    type Arguments = AlmostInfiniteClark2DtDispersalArguments;
    type Error = !;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G> for AlmostInfiniteClark2DtDispersalScenario<M, G> {
    type Decomposition = RadialDecomposition;
    type DecompositionAuxiliary = ();
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        AlmostInfiniteClark2DtDispersalSampler<M, G>;
    type Habitat = AlmostInfiniteHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> =
        SingletonDemesLineageStore<M, Self::Habitat>;
    type OriginSampler<'h, I: Iterator<Item = u64>> = AlmostInfiniteRectangleOriginSampler<'h, M, I> where G: 'h;
    type OriginSamplerAuxiliary = (LandscapeExtent,);
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = AlmostInfiniteHabitat::default();
        let dispersal_sampler =
            AlmostInfiniteClark2DtDispersalSampler::new(args.shape_u, args.tail_p);
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
        AlmostInfiniteRectangleOriginSampler::new(pre_sampler, habitat, sample)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        RadialDecomposition::new(subdomain)
    }
}

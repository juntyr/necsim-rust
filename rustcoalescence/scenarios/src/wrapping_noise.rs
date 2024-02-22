use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

use necsim_core::{
    cogs::{DispersalSampler, LineageStore, MathsCore, RngCore},
    landscape::LandscapeExtent,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::wrapping_noise::WrappingNoiseApproximateNormalDispersalSampler,
        habitat::wrapping_noise::WrappingNoiseHabitat,
        lineage_store::coherent::globally::singleton_demes::SingletonDemesLineageStore,
        origin_sampler::{
            pre_sampler::OriginPreSampler, wrapping_noise::WrappingNoiseOriginSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::radial::RadialDecomposition,
};

use crate::{Scenario, ScenarioParameters};

#[allow(clippy::module_name_repetitions)]
pub struct WrappingNoiseScenario<M: MathsCore, G: RngCore<M>> {
    sample: LandscapeExtent,

    habitat: WrappingNoiseHabitat<M>,
    dispersal_sampler: WrappingNoiseApproximateNormalDispersalSampler<M, G>,
    turnover_rate: UniformTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "WrappingNoise")]
pub struct WrappingNoiseArguments {
    pub seed: i64,
    pub coverage: ClosedUnitF64,
    pub scale: PositiveUnitF64,
    pub persistence: PositiveUnitF64,
    pub octaves: NonZeroUsize,
    pub sample: LandscapeExtent,
    pub sigma: NonNegativeF64,
}

impl<M: MathsCore, G: RngCore<M>> ScenarioParameters for WrappingNoiseScenario<M, G> {
    type Arguments = WrappingNoiseArguments;
    type Error = !;
}

impl<M: MathsCore, G: RngCore<M>> Scenario<M, G> for WrappingNoiseScenario<M, G> {
    type Decomposition = RadialDecomposition;
    type DecompositionAuxiliary = ();
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> =
        WrappingNoiseApproximateNormalDispersalSampler<M, G>;
    type Habitat = WrappingNoiseHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> =
        SingletonDemesLineageStore<M, Self::Habitat>;
    type OriginSampler<'h, I: Iterator<Item = u64>> = WrappingNoiseOriginSampler<'h, M, I> where G: 'h;
    type OriginSamplerAuxiliary = (LandscapeExtent,);
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = WrappingNoiseHabitat::new(
            args.seed,
            args.coverage,
            args.scale,
            args.persistence,
            args.octaves,
        );
        let dispersal_sampler = WrappingNoiseApproximateNormalDispersalSampler::new(args.sigma);
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

    #[allow(clippy::type_complexity)]
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
    ) -> Self::OriginSampler<'_, I>
    where
        G: 'h,
    {
        WrappingNoiseOriginSampler::new(pre_sampler, habitat, sample)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        RadialDecomposition::new(subdomain)
    }
}

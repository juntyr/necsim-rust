use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use necsim_core::cogs::{Habitat, LineageStore, MathsCore, RngCore};
use necsim_core_bond::{OpenClosedUnitF64 as PositiveUnitF64, PositiveF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::{
            almost_infinite_clark2dt::AlmostInfiniteClark2DtDispersalSampler,
            almost_infinite_downscaled::AlmostInfiniteDownscaledDispersalSampler,
        },
        habitat::almost_infinite::{
            downscaled::{AlmostInfiniteDownscaledHabitat, Log2U16},
            AlmostInfiniteHabitat,
        },
        lineage_store::coherent::globally::singleton_demes::SingletonDemesLineageStore,
        origin_sampler::{
            pre_sampler::OriginPreSampler, singleton_demes::SingletonDemesOriginSampler,
        },
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::uniform::UniformTurnoverRate,
    },
    decomposition::radial::RadialDecomposition,
};

use crate::{Scenario, ScenarioCogs, ScenarioParameters};

use super::Sample;

#[allow(clippy::module_name_repetitions, clippy::empty_enum)]
pub struct AlmostInfiniteDownscaledScenario<
    M: MathsCore,
    G: RngCore<M>,
    O: Scenario<M, G, Habitat = AlmostInfiniteHabitat<M>>,
> {
    _marker: PhantomData<(M, G, O)>,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "AlmostInfiniteDownscaled")]
pub struct AlmostInfiniteDownscaledArguments {
    pub sample: Sample,
    pub downscale_x: Log2U16,
    pub downscale_y: Log2U16,
}

impl<M: MathsCore, G: RngCore<M>, O: Scenario<M, G, Habitat = AlmostInfiniteHabitat<M>>>
    ScenarioParameters for AlmostInfiniteDownscaledScenario<M, G, O>
{
    type Arguments = AlmostInfiniteDownscaledArguments;
    type Error = !;
}

impl<M: MathsCore, G: RngCore<M>, O: Scenario<M, G, Habitat = AlmostInfiniteHabitat<M>>>
    Scenario<M, G> for AlmostInfiniteDownscaledScenario<M, G, O>
{
    type Decomposition = RadialDecomposition;
    type DecompositionAuxiliary = ();
    type DispersalSampler = AlmostInfiniteDownscaledDispersalSampler<M, G, O::DispersalSampler>;
    type Habitat = AlmostInfiniteDownscaledHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = SingletonDemesOriginSampler<'h, M, Self::Habitat, I> where G: 'h;
    type OriginSamplerAuxiliary = (Sample,);
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = UniformTurnoverRate;

    fn new(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<ScenarioCogs<M, G, Self>, Self::Error> {
        let habitat = AlmostInfiniteDownscaledHabitat::new(args.downscale_x, args.downscale_y);
        let dispersal_sampler = AlmostInfiniteDownscaledDispersalSampler::new(todo!());
        let turnover_rate = UniformTurnoverRate::default();
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        Ok(ScenarioCogs {
            habitat,
            dispersal_sampler,
            turnover_rate,
            speciation_probability,
            origin_sampler_auxiliary: (args.sample,),
            decomposition_auxiliary: (),
            _marker: PhantomData::<(M, G, Self)>,
        })
    }

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<M, I>,
        (sample,): Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h,
    {
        sample.into_origin_sampler(habitat, pre_sampler)
    }

    fn decompose(
        _habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        RadialDecomposition::new(subdomain)
    }
}

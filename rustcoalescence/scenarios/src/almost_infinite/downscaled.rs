use std::marker::PhantomData;

use serde::{Deserialize, Serialize};

use necsim_core::cogs::{LineageStore, MathsCore, RngCore, SpeciationProbability, TurnoverRate};
use necsim_core_bond::OpenClosedUnitF64 as PositiveUnitF64;
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    cogs::{
        dispersal_sampler::almost_infinite_downscaled::AlmostInfiniteDownscaledDispersalSampler,
        habitat::almost_infinite::{
            downscaled::{AlmostInfiniteDownscaledHabitat, Log2U16},
            AlmostInfiniteHabitat,
        },
        origin_sampler::{
            pre_sampler::OriginPreSampler,
            singleton_demes::downscaled::AlmostInfiniteDownscaledOriginSampler,
        },
    },
    decomposition::Decomposition,
};

use crate::{Scenario, ScenarioCogs, ScenarioParameters};

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
#[serde(bound = "O::Arguments: serde::Serialize + serde::de::DeserializeOwned")]
pub struct AlmostInfiniteDownscaledArguments<O: ScenarioParameters> {
    #[serde(flatten)]
    pub args: O::Arguments,
    pub downscale_x: Log2U16,
    pub downscale_y: Log2U16,
}

impl<
        M: MathsCore,
        G: RngCore<M>,
        O: Scenario<
            M,
            G,
            Habitat = AlmostInfiniteHabitat<M>,
            Decomposition: Decomposition<M, AlmostInfiniteDownscaledHabitat<M>>,
            SpeciationProbability: SpeciationProbability<M, AlmostInfiniteDownscaledHabitat<M>>,
            TurnoverRate: TurnoverRate<M, AlmostInfiniteDownscaledHabitat<M>>,
        >,
    > ScenarioParameters for AlmostInfiniteDownscaledScenario<M, G, O>
{
    type Arguments = AlmostInfiniteDownscaledArguments<O>;
    type Error = O::Error;
}

impl<
        M: MathsCore,
        G: RngCore<M>,
        O: Scenario<
            M,
            G,
            Habitat = AlmostInfiniteHabitat<M>,
            Decomposition: Decomposition<M, AlmostInfiniteDownscaledHabitat<M>>,
            SpeciationProbability: SpeciationProbability<M, AlmostInfiniteDownscaledHabitat<M>>,
            TurnoverRate: TurnoverRate<M, AlmostInfiniteDownscaledHabitat<M>>,
        >,
    > Scenario<M, G> for AlmostInfiniteDownscaledScenario<M, G, O>
{
    type Decomposition = O::Decomposition;
    type DecompositionAuxiliary = O::DecompositionAuxiliary;
    type DispersalSampler = AlmostInfiniteDownscaledDispersalSampler<M, G, O::DispersalSampler>;
    type Habitat = AlmostInfiniteDownscaledHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = AlmostInfiniteDownscaledOriginSampler<'h, M, O::OriginSampler<'h, I>> where G: 'h, O: 'h;
    type OriginSamplerAuxiliary = O::OriginSamplerAuxiliary;
    type SpeciationProbability = O::SpeciationProbability;
    type TurnoverRate = O::TurnoverRate;

    fn new(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<ScenarioCogs<M, G, Self>, Self::Error> {
        let ScenarioCogs {
            habitat,
            dispersal_sampler,
            turnover_rate,
            speciation_probability,
            origin_sampler_auxiliary,
            decomposition_auxiliary,
            _marker,
        } = O::new(args.args, speciation_probability_per_generation)?;

        let habitat = AlmostInfiniteDownscaledHabitat::new_with_habitat(
            habitat,
            args.downscale_x,
            args.downscale_y,
        );
        let dispersal_sampler = AlmostInfiniteDownscaledDispersalSampler::new(dispersal_sampler);

        Ok(ScenarioCogs {
            habitat,
            dispersal_sampler,
            turnover_rate,
            speciation_probability,
            origin_sampler_auxiliary,
            decomposition_auxiliary,
            _marker: PhantomData::<(M, G, Self)>,
        })
    }

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<M, I>,
        auxiliary: Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h,
    {
        AlmostInfiniteDownscaledOriginSampler::new(
            O::sample_habitat(habitat.unscaled(), pre_sampler, auxiliary),
            habitat,
        )
    }

    fn decompose(
        habitat: &Self::Habitat,
        subdomain: Partition,
        auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        O::decompose(habitat.unscaled(), subdomain, auxiliary)
    }
}

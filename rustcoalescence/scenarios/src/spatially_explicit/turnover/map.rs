use std::{convert::TryFrom, marker::PhantomData, path::PathBuf};

use serde::{Deserialize, Serialize, Serializer};

use necsim_core::cogs::{
    distribution::IndexU64, DispersalSampler, Habitat, LineageStore, MathsCore, Rng, Samples,
};
use necsim_core_bond::{NonNegativeF64, OpenClosedUnitF64 as PositiveUnitF64};
use necsim_partitioning_core::partition::Partition;

use necsim_impls_no_std::{
    array2d::Array2D,
    cogs::{
        dispersal_sampler::in_memory::{
            contract::explicit_in_memory_dispersal_check_contract, InMemoryDispersalSampler,
        },
        habitat::in_memory::InMemoryHabitat,
        origin_sampler::{in_memory::InMemoryOriginSampler, pre_sampler::OriginPreSampler},
        speciation_probability::uniform::UniformSpeciationProbability,
        turnover_rate::in_memory::{InMemoryTurnoverRate, InMemoryTurnoverRateError},
    },
    decomposition::equal::EqualDecomposition,
};

use necsim_impls_std::cogs::dispersal_sampler::in_memory::error::InMemoryDispersalSamplerError;

use crate::{Scenario, ScenarioParameters};

use super::super::maps::{self, MapLoadingMode};

#[allow(clippy::module_name_repetitions, clippy::enum_variant_names)]
#[derive(thiserror::Error, displaydoc::Display, Debug)]
pub enum SpatiallyExplicitTurnoverMapScenarioError {
    /// invalid habitat map: no habitable locations
    EmptyHabitatMap,
    /// invalid dispersal map: {0}
    DispersalMap(InMemoryDispersalSamplerError),
    /// invalid turnover map: {0}
    TurnoverMap(InMemoryTurnoverRateError),
}

#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitTurnoverMapScenario<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> {
    habitat: InMemoryHabitat<M>,
    dispersal_map: Array2D<NonNegativeF64>,
    turnover_rate: InMemoryTurnoverRate,
    speciation_probability: UniformSpeciationProbability,
    _marker: PhantomData<G>,
}

impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> ScenarioParameters
    for SpatiallyExplicitTurnoverMapScenario<M, G>
{
    type Arguments = SpatiallyExplicitTurnoverMapArguments;
    type Error = SpatiallyExplicitTurnoverMapScenarioError;
}

impl<M: MathsCore, G: Rng<M> + Samples<M, IndexU64>> Scenario<M, G>
    for SpatiallyExplicitTurnoverMapScenario<M, G>
{
    type Decomposition = EqualDecomposition<M, Self::Habitat>;
    type DecompositionAuxiliary = ();
    type DispersalSampler<D: DispersalSampler<M, Self::Habitat, G>> = D;
    type Habitat = InMemoryHabitat<M>;
    type LineageStore<L: LineageStore<M, Self::Habitat>> = L;
    type OriginSampler<'h, I: Iterator<Item = u64>> = InMemoryOriginSampler<'h, M, I> where G: 'h;
    type OriginSamplerAuxiliary = ();
    type SpeciationProbability = UniformSpeciationProbability;
    type TurnoverRate = InMemoryTurnoverRate;

    fn initialise(
        args: Self::Arguments,
        speciation_probability_per_generation: PositiveUnitF64,
    ) -> Result<Self, Self::Error> {
        let habitat = InMemoryHabitat::try_new(args.habitat_map)
            .ok_or(SpatiallyExplicitTurnoverMapScenarioError::EmptyHabitatMap)?;
        let turnover_rate = InMemoryTurnoverRate::new(args.turnover_map, &habitat)
            .map_err(SpatiallyExplicitTurnoverMapScenarioError::TurnoverMap)?;
        let speciation_probability =
            UniformSpeciationProbability::new(speciation_probability_per_generation.into());

        let habitat_extent = habitat.get_extent();
        let habitat_area =
            usize::from(habitat_extent.width()) * usize::from(habitat_extent.height());

        if args.dispersal_map.num_rows() != habitat_area
            || args.dispersal_map.num_columns() != habitat_area
        {
            return Err(SpatiallyExplicitTurnoverMapScenarioError::DispersalMap(
                InMemoryDispersalSamplerError::InconsistentDispersalMapSize,
            ));
        }

        if !explicit_in_memory_dispersal_check_contract(&args.dispersal_map, &habitat) {
            return Err(SpatiallyExplicitTurnoverMapScenarioError::DispersalMap(
                InMemoryDispersalSamplerError::InconsistentDispersalProbabilities,
            ));
        }

        Ok(Self {
            habitat,
            dispersal_map: args.dispersal_map,
            turnover_rate,
            speciation_probability,
            _marker: PhantomData::<G>,
        })
    }

    fn build<D: InMemoryDispersalSampler<M, Self::Habitat, G>>(
        self,
    ) -> (
        Self::Habitat,
        Self::DispersalSampler<D>,
        Self::TurnoverRate,
        Self::SpeciationProbability,
        Self::OriginSamplerAuxiliary,
        Self::DecompositionAuxiliary,
    ) {
        let dispersal_sampler = D::unchecked_new(&self.dispersal_map, &self.habitat);

        (
            self.habitat,
            dispersal_sampler,
            self.turnover_rate,
            self.speciation_probability,
            (),
            (),
        )
    }

    fn sample_habitat<'h, I: Iterator<Item = u64>>(
        habitat: &'h Self::Habitat,
        pre_sampler: OriginPreSampler<I>,
        _auxiliary: Self::OriginSamplerAuxiliary,
    ) -> Self::OriginSampler<'h, I>
    where
        G: 'h,
    {
        InMemoryOriginSampler::new(pre_sampler, habitat)
    }

    fn decompose(
        habitat: &Self::Habitat,
        subdomain: Partition,
        _auxiliary: Self::DecompositionAuxiliary,
    ) -> Self::Decomposition {
        match EqualDecomposition::weight(habitat, subdomain) {
            Ok(decomposition) => decomposition,
            Err(decomposition) => {
                warn!(
                    "Spatially explicit habitat of size {}x{} could not be partitioned into {} \
                     partition(s).",
                    habitat.get_extent().width(),
                    habitat.get_extent().height(),
                    subdomain.size().get(),
                );

                decomposition
            },
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "SpatiallyExplicitTurnoverMapArgumentsRaw")]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyExplicitTurnoverMapArguments {
    pub habitat_path: PathBuf,
    pub habitat_map: Array2D<u32>,
    pub dispersal_path: PathBuf,
    pub dispersal_map: Array2D<NonNegativeF64>,
    pub turnover_path: PathBuf,
    pub turnover_map: Array2D<NonNegativeF64>,
    pub loading_mode: MapLoadingMode,
}

impl Serialize for SpatiallyExplicitTurnoverMapArguments {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        SpatiallyExplicitTurnoverMapArgumentsRaw {
            habitat_map: self.habitat_path.clone(),
            dispersal_map: self.dispersal_path.clone(),
            turnover_map: self.turnover_path.clone(),
            loading_mode: self.loading_mode,
        }
        .serialize(serializer)
    }
}

impl TryFrom<SpatiallyExplicitTurnoverMapArgumentsRaw> for SpatiallyExplicitTurnoverMapArguments {
    type Error = String;

    fn try_from(raw: SpatiallyExplicitTurnoverMapArgumentsRaw) -> Result<Self, Self::Error> {
        info!(
            "Starting to load the dispersal map {:?} ...",
            &raw.dispersal_map
        );

        let mut dispersal_map = maps::load_dispersal_map(&raw.dispersal_map, raw.loading_mode)
            .map_err(|err| format!("{err:?}"))?;

        info!(
            "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
            &raw.dispersal_map,
            dispersal_map.num_columns(),
            dispersal_map.num_rows()
        );

        info!(
            "Starting to load the turnover map {:?} ...",
            &raw.turnover_map
        );

        let turnover_map = maps::load_turnover_map(&raw.turnover_map, raw.loading_mode)
            .map_err(|err| format!("{err:?}"))?;

        info!(
            "Successfully loaded the turnover map {:?} with dimensions {}x{} [cols x rows].",
            &raw.turnover_map,
            turnover_map.num_columns(),
            turnover_map.num_rows()
        );

        info!(
            "Starting to load the habitat map {:?} ...",
            &raw.habitat_map
        );

        let habitat_map = maps::load_habitat_map(
            &raw.habitat_map,
            Some(&turnover_map),
            &mut dispersal_map,
            raw.loading_mode,
        )
        .map_err(|err| format!("{err:?}"))?;

        info!(
            "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
            &raw.habitat_map,
            habitat_map.num_columns(),
            habitat_map.num_rows()
        );

        Ok(SpatiallyExplicitTurnoverMapArguments {
            habitat_path: raw.habitat_map,
            habitat_map,
            dispersal_path: raw.dispersal_map,
            dispersal_map,
            turnover_path: raw.turnover_map,
            turnover_map,
            loading_mode: raw.loading_mode,
        })
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(rename = "SpatiallyExplicitTurnoverMapArguments")]
struct SpatiallyExplicitTurnoverMapArgumentsRaw {
    #[serde(rename = "habitat", alias = "habitat_map")]
    habitat_map: PathBuf,

    #[serde(rename = "dispersal", alias = "dispersal_map")]
    dispersal_map: PathBuf,

    #[serde(rename = "turnover", alias = "turnover_map")]
    turnover_map: PathBuf,

    #[serde(default)]
    #[serde(rename = "mode", alias = "loading_mode")]
    loading_mode: MapLoadingMode,
}

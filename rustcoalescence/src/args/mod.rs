use std::{convert::TryFrom, path::PathBuf};

use array2d::Array2D;
use serde::Deserialize;
use structopt::StructOpt;

mod parse;

use necsim_impls_std::bounded::{NonNegativeF64, ZeroExclOneInclF64, ZeroInclOneInclF64};

#[derive(Debug, StructOpt)]
#[allow(clippy::module_name_repetitions)]
pub enum RustcoalescenceArgs {
    Simulate(SimulateCommandArgs),
    Replay(ReplayCommandArgs),
}

#[derive(Debug, StructOpt)]
#[allow(clippy::module_name_repetitions)]
#[structopt(template("{bin} {version}\n\nUSAGE:\n    {usage} args..\n\n{all-args}"))]
#[structopt(setting(structopt::clap::AppSettings::AllowLeadingHyphen))]
pub struct SimulateCommandArgs {
    #[structopt(hidden(true))]
    pub args: Vec<String>,
}

#[derive(Debug, StructOpt)]
#[allow(clippy::module_name_repetitions)]
pub struct ReplayCommandArgs {
    #[structopt(parse(from_os_str))]
    pub events: Vec<PathBuf>,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(from = "SimulateArgsRaw")]
pub struct SimulateArgs {
    pub common: CommonArgs,

    #[serde(alias = "log")]
    pub event_log: Option<PathBuf>,

    pub scenario: Scenario,
}

impl From<SimulateArgsRaw> for SimulateArgs {
    fn from(raw: SimulateArgsRaw) -> Self {
        Self {
            common: CommonArgs {
                speciation_probability_per_generation: raw.speciation_probability_per_generation,
                sample_percentage: raw.sample_percentage,
                seed: raw.seed,
                algorithm: raw.algorithm,
            },
            event_log: raw.event_log,
            scenario: raw.scenario,
        }
    }
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
struct SimulateArgsRaw {
    #[serde(alias = "speciation")]
    pub speciation_probability_per_generation: ZeroExclOneInclF64,

    #[serde(alias = "sample")]
    pub sample_percentage: ZeroInclOneInclF64,

    pub seed: u64,

    pub algorithm: Algorithm,

    #[serde(alias = "log")]
    pub event_log: Option<PathBuf>,

    pub scenario: Scenario,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CommonArgs {
    pub speciation_probability_per_generation: ZeroExclOneInclF64,
    pub sample_percentage: ZeroInclOneInclF64,
    pub seed: u64,
    pub algorithm: Algorithm,
}

#[derive(Debug, Deserialize)]
#[non_exhaustive]
#[allow(clippy::empty_enum)]
pub enum Algorithm {
    #[cfg(feature = "necsim-classical")]
    Classical,
    #[cfg(feature = "necsim-gillespie")]
    Gillespie,
    #[cfg(feature = "necsim-skipping-gillespie")]
    SkippingGillespie(necsim_skipping_gillespie::SkippingGillespieArguments),
    #[cfg(feature = "necsim-cuda")]
    #[serde(alias = "CUDA")]
    Cuda(necsim_cuda::CudaArguments),
    #[cfg(feature = "necsim-independent")]
    Independent(necsim_independent::IndependentArguments),
}

#[derive(Debug, Deserialize)]
#[serde(from = "ScenarioRaw")]
pub enum Scenario {
    InMemory(InMemoryArgs),
    NonSpatial(NonSpatialArgs),
    SpatiallyImplicit(SpatiallyImplicitArgs),
    AlmostInfinite(AlmostInfiniteArgs),
}

impl From<ScenarioRaw> for Scenario {
    fn from(raw: ScenarioRaw) -> Self {
        match raw {
            ScenarioRaw::InMemory(args) => Scenario::InMemory(args),
            ScenarioRaw::NonSpatial(args) => {
                if args.spatial {
                    let habitat_map =
                        Array2D::filled_with(args.deme, args.area.1 as usize, args.area.0 as usize);

                    let total_area = (args.area.0 as usize) * (args.area.1 as usize);

                    let dispersal_map = Array2D::filled_with(1.0_f64, total_area, total_area);

                    Scenario::InMemory(InMemoryArgs {
                        habitat_map,
                        dispersal_map,
                    })
                } else {
                    Scenario::NonSpatial(NonSpatialArgs {
                        area: args.area,
                        deme: args.deme,
                    })
                }
            },
            ScenarioRaw::SpatiallyImplicit(args) => Scenario::SpatiallyImplicit(args),
            ScenarioRaw::AlmostInfinite(args) => Scenario::AlmostInfinite(args),
        }
    }
}

#[derive(Debug, Deserialize)]
enum ScenarioRaw {
    InMemory(InMemoryArgs),
    NonSpatial(NonSpatialArgsRaw),
    SpatiallyImplicit(SpatiallyImplicitArgs),
    AlmostInfinite(AlmostInfiniteArgs),
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "InMemory")]
#[serde(try_from = "InMemoryArgsRaw")]
pub struct InMemoryArgs {
    pub habitat_map: Array2D<u32>,
    pub dispersal_map: Array2D<f64>,
}

impl TryFrom<InMemoryArgsRaw> for InMemoryArgs {
    type Error = anyhow::Error;

    fn try_from(raw: InMemoryArgsRaw) -> Result<Self, Self::Error> {
        info!(
            "Starting to load the dispersal map {:?} ...",
            &raw.dispersal_map
        );

        let dispersal_map = crate::maps::load_dispersal_map(&raw.dispersal_map, raw.strict_load)?;

        info!(
            "Successfully loaded the dispersal map {:?} with dimensions {}x{} [cols x rows].",
            &raw.dispersal_map,
            dispersal_map.num_columns(),
            dispersal_map.num_rows()
        );

        info!(
            "Starting to load the habitat map {:?} ...",
            &raw.habitat_map
        );

        let habitat_map =
            crate::maps::load_habitat_map(&raw.habitat_map, &dispersal_map, raw.strict_load)?;

        info!(
            "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
            &raw.habitat_map,
            habitat_map.num_columns(),
            habitat_map.num_rows()
        );

        Ok(Self {
            habitat_map,
            dispersal_map,
        })
    }
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "InMemory")]
struct InMemoryArgsRaw {
    #[serde(alias = "habitat")]
    habitat_map: PathBuf,

    #[serde(alias = "dispersal")]
    dispersal_map: PathBuf,

    #[serde(default)]
    #[serde(alias = "strict")]
    strict_load: bool,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialArgs {
    pub area: (u32, u32),
    pub deme: u32,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
struct NonSpatialArgsRaw {
    pub area: (u32, u32),
    pub deme: u32,

    #[serde(default)]
    pub spatial: bool,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitArgs {
    pub local_area: (u32, u32),
    pub local_deme: u32,
    pub meta_area: (u32, u32),
    pub meta_deme: u32,

    #[serde(alias = "migration")]
    pub migration_probability_per_generation: ZeroExclOneInclF64,

    #[serde(default)]
    pub dynamic_meta: bool,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
pub struct AlmostInfiniteArgs {
    pub radius: u32,
    pub sigma: NonNegativeF64,
}

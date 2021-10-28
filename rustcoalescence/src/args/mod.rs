use std::{convert::TryFrom, fmt, ops::Deref, path::PathBuf};

use serde::{de::Deserializer, Deserialize};
use serde_state::DeserializeState;
use structopt::StructOpt;

use necsim_core_bond::{ClosedUnitF64, Partition, PositiveUnitF64};

use necsim_impls_no_std::array2d::Array2D;
use necsim_impls_std::event_log::{recorder::EventLogRecorder, replay::EventLogReplay};

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteArguments, non_spatial::NonSpatialArguments,
    spatially_explicit::InMemoryArguments, spatially_implicit::SpatiallyImplicitArguments,
};

#[cfg(any(
    feature = "rustcoalescence-algorithms-monolithic",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
use rustcoalescence_algorithms::AlgorithmArguments;

use necsim_plugins_core::import::{AnyReporterPluginVec, ReporterPluginLibrary};

mod parse;

#[derive(Debug, StructOpt)]
#[allow(clippy::module_name_repetitions)]
pub enum RustcoalescenceArgs {
    Simulate(CommandArgs),
    Replay(CommandArgs),
}

#[derive(Debug, StructOpt)]
#[allow(clippy::module_name_repetitions)]
#[structopt(template("{bin} {version}\n\nUSAGE:\n    {usage} args..\n\n{all-args}"))]
#[structopt(setting(structopt::clap::AppSettings::AllowLeadingHyphen))]
pub struct CommandArgs {
    #[structopt(hidden(true))]
    pub args: Vec<String>,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct SimulateArgs {
    pub common: CommonArgs,
    pub scenario: Scenario,
    pub algorithm: Algorithm,
    pub partitioning: Partitioning,
    pub event_log: Option<EventLogRecorder>,
    pub reporters: AnyReporterPluginVec,
}

impl<'de> DeserializeState<'de, Partition> for SimulateArgs {
    fn deserialize_state<D>(seed: &mut Partition, deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = SimulateArgsRaw::deserialize_state(seed, deserializer)?;

        Ok(Self {
            common: CommonArgs {
                speciation_probability_per_generation: raw.speciation_probability_per_generation,
                sample_percentage: raw.sample_percentage,
                rng: raw.rng,
            },
            scenario: raw.scenario,
            algorithm: raw.algorithm,
            partitioning: raw.partitioning,
            event_log: raw.event_log,
            reporters: raw.reporters.into_iter().flatten().collect(),
        })
    }
}

#[derive(DeserializeState)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(deserialize_state = "Partition")]
struct SimulateArgsRaw {
    #[serde(alias = "speciation")]
    speciation_probability_per_generation: PositiveUnitF64,

    #[serde(alias = "sample")]
    sample_percentage: ClosedUnitF64,

    #[serde(alias = "randomness")]
    #[serde(default)]
    rng: Rng,

    scenario: Scenario,

    #[serde(deserialize_state)]
    algorithm: Algorithm,

    #[serde(default)]
    partitioning: Partitioning,

    #[serde(alias = "log")]
    #[serde(default)]
    #[serde(deserialize_state_with = "deserialize_state_event_log")]
    event_log: Option<EventLogRecorder>,

    reporters: Vec<ReporterPluginLibrary>,
}

fn deserialize_state_event_log<'de, D>(
    partition: &mut Partition,
    deserializer: D,
) -> Result<Option<EventLogRecorder>, D::Error>
where
    D: Deserializer<'de>,
{
    use serde::de::Error;

    let event_log_path = <Option<PathBuf>>::deserialize(deserializer)?;

    match event_log_path {
        Some(mut event_log_path) => {
            event_log_path.push(partition.rank().to_string());

            let event_log = EventLogRecorder::try_new(&event_log_path).map_err(D::Error::custom)?;

            Ok(Some(event_log))
        },
        None => Ok(None),
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CommonArgs {
    pub speciation_probability_per_generation: PositiveUnitF64,
    pub sample_percentage: ClosedUnitF64,
    pub rng: Rng,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Rng {
    Entropy,
    Seed(u64),
    Sponge(Base32String),
    State(Base32String),
    StateElseSponge(Base32String),
}

impl Default for Rng {
    fn default() -> Self {
        Self::Entropy
    }
}

#[derive(Clone)]
#[repr(transparent)]
pub struct Base32String(Box<[u8]>);

impl Base32String {
    #[must_use]
    #[allow(dead_code)]
    pub fn new(bytes: &[u8]) -> Self {
        Self(bytes.to_vec().into_boxed_slice())
    }
}

impl fmt::Display for Base32String {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "{:?}",
            base32::encode(base32::Alphabet::Crockford, &self.0).to_ascii_lowercase()
        )
    }
}

impl fmt::Debug for Base32String {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "base32: {:?}",
            base32::encode(base32::Alphabet::Crockford, &self.0).to_ascii_lowercase()
        )
    }
}

impl<'de> Deserialize<'de> for Base32String {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        if let Some(data) = base32::decode(
            base32::Alphabet::Crockford,
            <&str>::deserialize(deserializer)?,
        ) {
            Ok(Self(data.into_boxed_slice()))
        } else {
            Err(serde::de::Error::custom(
                "Invalid Crockford's Base32 string: must only contain alphanumeric characters.",
            ))
        }
    }
}

impl Deref for Base32String {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize)]
pub enum Partitioning {
    Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning),
    #[cfg(feature = "necsim-partitioning-mpi")]
    #[serde(alias = "MPI")]
    Mpi(necsim_partitioning_mpi::MpiPartitioning),
}

impl Partitioning {
    pub fn is_root(&self) -> bool {
        use necsim_partitioning_core::Partitioning;

        match self {
            Self::Monolithic(partitioning) => partitioning.is_root(),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Self::Mpi(partitioning) => partitioning.is_root(),
        }
    }

    pub fn get_partition(&self) -> Partition {
        use necsim_partitioning_core::Partitioning;

        match self {
            Self::Monolithic(partitioning) => partitioning.get_partition(),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Self::Mpi(partitioning) => partitioning.get_partition(),
        }
    }
}

impl Default for Partitioning {
    fn default() -> Self {
        Self::Monolithic(necsim_partitioning_monolithic::MonolithicPartitioning::default())
    }
}

#[derive(Debug, DeserializeState)]
#[serde(deserialize_state = "Partition")]
pub enum Algorithm {
    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
    Classical(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_monolithic::classical::ClassicalAlgorithm as rustcoalescence_algorithms::AlgorithmArguments>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
    Gillespie(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_monolithic::gillespie::GillespieAlgorithm as AlgorithmArguments>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-monolithic")]
    SkippingGillespie(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_monolithic::skipping_gillespie::SkippingGillespieAlgorithm as AlgorithmArguments>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
    #[serde(alias = "CUDA")]
    Cuda(#[serde(deserialize_state)] <rustcoalescence_algorithms_cuda::CudaAlgorithm as AlgorithmArguments>::Arguments),
    #[cfg(feature = "rustcoalescence-algorithms-independent")]
    Independent(
        #[serde(deserialize_state)] <rustcoalescence_algorithms_independent::IndependentAlgorithm as AlgorithmArguments>::Arguments,
    ),
}

impl fmt::Display for Algorithm {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "necsim-classical")]
            Self::Classical => fmt.write_str("Classical"),
            #[cfg(feature = "necsim-gillespie")]
            Self::Gillespie => fmt.write_str("Gillespie"),
            #[cfg(feature = "necsim-skipping-gillespie")]
            Self::SkippingGillespie(_) => fmt.write_str("Skipping-Gillespie"),
            #[cfg(feature = "necsim-cuda")]
            Self::Cuda(_) => fmt.write_str("CUDA"),
            #[cfg(feature = "necsim-independent")]
            Self::Independent(_) => fmt.write_str("Independent"),
            _ => fmt.write_str("Unknown"),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(from = "ScenarioRaw")]
pub enum Scenario {
    SpatiallyExplicit(InMemoryArguments),
    NonSpatial(NonSpatialArguments),
    SpatiallyImplicit(SpatiallyImplicitArguments),
    AlmostInfinite(AlmostInfiniteArguments),
}

impl From<ScenarioRaw> for Scenario {
    fn from(raw: ScenarioRaw) -> Self {
        match raw {
            ScenarioRaw::SpatiallyExplicit(args) => {
                Scenario::SpatiallyExplicit(InMemoryArguments {
                    habitat_map: args.habitat_map,
                    dispersal_map: args.dispersal_map,
                })
            },
            ScenarioRaw::NonSpatial(args) => {
                if args.spatial {
                    let habitat_map =
                        Array2D::filled_with(args.deme, args.area.1 as usize, args.area.0 as usize);

                    let total_area = (args.area.0 as usize) * (args.area.1 as usize);

                    let dispersal_map = Array2D::filled_with(1.0_f64, total_area, total_area);

                    Scenario::SpatiallyExplicit(InMemoryArguments {
                        habitat_map,
                        dispersal_map,
                    })
                } else {
                    Scenario::NonSpatial(NonSpatialArguments {
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
    SpatiallyExplicit(InMemoryArgs),
    NonSpatial(NonSpatialArgsRaw),
    SpatiallyImplicit(SpatiallyImplicitArguments),
    AlmostInfinite(AlmostInfiniteArguments),
}

#[derive(Debug, Deserialize)]
#[serde(try_from = "InMemoryArgsRaw")]
struct InMemoryArgs {
    habitat_map: Array2D<u32>,
    dispersal_map: Array2D<f64>,
}

impl TryFrom<InMemoryArgsRaw> for InMemoryArgs {
    type Error = String;

    fn try_from(raw: InMemoryArgsRaw) -> Result<Self, Self::Error> {
        info!(
            "Starting to load the dispersal map {:?} ...",
            &raw.dispersal_map
        );

        let mut dispersal_map =
            crate::maps::load_dispersal_map(&raw.dispersal_map, raw.loading_mode)
                .map_err(|err| format!("{:?}", err))?;

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
            crate::maps::load_habitat_map(&raw.habitat_map, &mut dispersal_map, raw.loading_mode)
                .map_err(|err| format!("{:?}", err))?;

        info!(
            "Successfully loaded the habitat map {:?} with dimensions {}x{} [cols x rows].",
            &raw.habitat_map,
            habitat_map.num_columns(),
            habitat_map.num_rows()
        );

        Ok(InMemoryArgs {
            habitat_map,
            dispersal_map,
        })
    }
}

#[derive(Copy, Clone, Debug, Deserialize)]
pub enum MapLoadingMode {
    FixMe,
    OffByOne,
    Strict,
}

impl Default for MapLoadingMode {
    fn default() -> Self {
        Self::OffByOne
    }
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(rename = "SpatiallyExplicit")]
#[serde(deny_unknown_fields)]
struct InMemoryArgsRaw {
    #[serde(alias = "habitat")]
    habitat_map: PathBuf,

    #[serde(alias = "dispersal")]
    dispersal_map: PathBuf,

    #[serde(default)]
    #[serde(alias = "mode")]
    loading_mode: MapLoadingMode,
}

#[derive(Debug, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(rename = "NonSpatial")]
struct NonSpatialArgsRaw {
    pub area: (u32, u32),
    pub deme: u32,

    #[serde(default)]
    pub spatial: bool,
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ReplayArgs {
    pub log: EventLogReplay,
    pub reporters: AnyReporterPluginVec,
}

impl<'de> Deserialize<'de> for ReplayArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = ReplayArgsRaw::deserialize(deserializer)?;

        let log = raw.logs;
        let reporters = raw.reporters.into_iter().flatten().collect();

        let (report_speciation, report_dispersal) = match &reporters {
            AnyReporterPluginVec::IgnoreSpeciationIgnoreDispersalIgnoreProgress(..)
            | AnyReporterPluginVec::IgnoreSpeciationIgnoreDispersalReportProgress(..) => {
                (false, false)
            },
            AnyReporterPluginVec::IgnoreSpeciationReportDispersalIgnoreProgress(..)
            | AnyReporterPluginVec::IgnoreSpeciationReportDispersalReportProgress(..) => {
                (false, true)
            },
            AnyReporterPluginVec::ReportSpeciationIgnoreDispersalIgnoreProgress(..)
            | AnyReporterPluginVec::ReportSpeciationIgnoreDispersalReportProgress(..) => {
                (true, false)
            },
            AnyReporterPluginVec::ReportSpeciationReportDispersalIgnoreProgress(..)
            | AnyReporterPluginVec::ReportSpeciationReportDispersalReportProgress(..) => {
                (true, true)
            },
        };

        let valid = if report_speciation
            && !log.with_speciation()
            && report_dispersal
            && !log.with_dispersal()
        {
            Err(
                "The reporters require speciation and dispersal events, but the event log cannot \
                 provide either.",
            )
        } else if report_speciation && !log.with_speciation() {
            Err("The reporters require speciation events, but the event log cannot provide them.")
        } else if report_dispersal && !log.with_dispersal() {
            Err("The reporters require dispersal events, but the event log cannot provide them.")
        } else {
            Ok(())
        };

        match (valid, raw.mode) {
            (Ok(_), _) => Ok(Self { log, reporters }),
            (Err(error), ReplayMode::WarnOnly) => {
                warn!("{}", error);

                Ok(Self { log, reporters })
            },
            (Err(error), ReplayMode::Strict) => Err(serde::de::Error::custom(error)),
        }
    }
}

#[derive(Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
enum ReplayMode {
    Strict,
    WarnOnly,
}

impl Default for ReplayMode {
    fn default() -> Self {
        Self::Strict
    }
}

#[derive(Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
struct ReplayArgsRaw {
    logs: EventLogReplay,
    #[serde(default)]
    mode: ReplayMode,
    reporters: Vec<ReporterPluginLibrary>,
}

use std::{fmt, ops::Deref};

use serde::{de::Deserializer, ser::SerializeStruct, Deserialize, Serialize, Serializer};
use serde_state::DeserializeState;
use structopt::StructOpt;

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, Partition, PositiveUnitF64};

use necsim_impls_std::{
    event_log::{recorder::EventLogRecorder, replay::EventLogReplay},
    lineage_file::loader::LineageFileLoader,
};

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteArguments, non_spatial::NonSpatialArguments,
    spatially_explicit::SpatiallyExplicitArguments, spatially_implicit::SpatiallyImplicitArguments,
};

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
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
    pub pause: Option<Pause>,
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
            pause: raw.pause,
        })
    }
}

#[derive(DeserializeState)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
#[serde(deserialize_state = "Partition")]
#[serde(rename = "Simulate")]
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

    #[serde(default)]
    #[serde(deserialize_state)]
    pause: Option<Pause>,
}

impl Serialize for SimulateArgs {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut args = serializer.serialize_struct("Simulate", 9)?;

        // Notes:
        // - sample should be set to 100% for resuming a paused simulation
        // - pause should be set to None for resuming a paused simulation
        // - serialization could be used to debug print a normalised config
        // - serialization will be used to get the pause config

        args.serialize_field(
            "speciation",
            &self.common.speciation_probability_per_generation,
        )?;
        args.serialize_field("sample", &self.common.sample_percentage)?;
        args.serialize_field("rng", &self.common.rng)?;
        args.serialize_field("scenario", &self.scenario)?;
        args.serialize_field("algorithm", &self.algorithm)?;
        args.serialize_field("partitioning", &self.partitioning)?;
        args.serialize_field("log", &self.event_log)?;
        args.serialize_field("reporters", &self.reporters)?;
        args.serialize_field("pause", &self.pause)?;

        args.end()
    }
}

fn deserialize_state_event_log<'de, D>(
    partition: &mut Partition,
    deserializer: D,
) -> Result<Option<EventLogRecorder>, D::Error>
where
    D: Deserializer<'de>,
{
    let event_log = match <Option<EventLogRecorder>>::deserialize(deserializer)? {
        Some(event_log) => event_log,
        None => return Ok(None),
    };

    if partition.size().get() <= 1 {
        return Ok(Some(event_log));
    }

    let mut directory = event_log.directory().to_owned();
    directory.push(partition.rank().to_string());

    match event_log.r#move(&directory) {
        Ok(event_log) => Ok(Some(event_log)),
        Err(err) => Err(serde::de::Error::custom(err)),
    }
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct CommonArgs {
    pub speciation_probability_per_generation: PositiveUnitF64,
    pub sample_percentage: ClosedUnitF64,
    pub rng: Rng,
}

#[derive(Debug, Serialize, Deserialize)]
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

impl Serialize for Base32String {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        base32::encode(base32::Alphabet::Crockford, &self.0)
            .to_ascii_lowercase()
            .serialize(serializer)
    }
}

impl Deref for Base32String {
    type Target = [u8];

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Serialize, Deserialize)]
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
    #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
    #[serde(alias = "Gillespie")]
    Classical(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::classical::ClassicalAlgorithm as rustcoalescence_algorithms::AlgorithmArguments>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
    #[serde(alias = "SkippingGillespie")]
    EventSkipping(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::event_skipping::EventSkippingAlgorithm as AlgorithmArguments>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
    #[serde(alias = "CUDA")]
    Cuda(#[serde(deserialize_state)] <rustcoalescence_algorithms_cuda::CudaAlgorithm as AlgorithmArguments>::Arguments),
    #[cfg(feature = "rustcoalescence-algorithms-independent")]
    Independent(
        #[serde(deserialize_state)] <rustcoalescence_algorithms_independent::IndependentAlgorithm as AlgorithmArguments>::Arguments,
    ),
}

impl Serialize for Algorithm {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[allow(unreachable_patterns)]
        match self {
            #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
            Self::Classical(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 0, "Classical", args)
            },
            #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
            Self::EventSkipping(args) => serializer.serialize_newtype_variant(
                stringify!(Algorithm),
                1,
                "EventSkipping",
                args,
            ),
            #[cfg(feature = "rustcoalescence-algorithms-cuda")]
            Self::Cuda(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 2, "CUDA", args)
            },
            #[cfg(feature = "rustcoalescence-algorithms-independent")]
            Self::Independent(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 3, "Independent", args)
            },
            _ => {
                std::mem::drop(serializer);

                Err(serde::ser::Error::custom(
                    "rustcoalescence must be compiled to support at least one algorithm",
                ))
            },
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Scenario {
    SpatiallyExplicit(SpatiallyExplicitArguments),
    NonSpatial(NonSpatialArguments),
    SpatiallyImplicit(SpatiallyImplicitArguments),
    AlmostInfinite(AlmostInfiniteArguments),
}

#[derive(Debug)]
#[allow(clippy::module_name_repetitions)]
pub struct ReplayArgs {
    pub event_log: EventLogReplay,
    pub reporters: AnyReporterPluginVec,
}

impl<'de> Deserialize<'de> for ReplayArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = ReplayArgsRaw::deserialize(deserializer)?;

        let event_log = raw.event_log;
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
            && !event_log.with_speciation()
            && report_dispersal
            && !event_log.with_dispersal()
        {
            Err(
                "The reporters require speciation and dispersal events, but the event log cannot \
                 provide either.",
            )
        } else if report_speciation && !event_log.with_speciation() {
            Err("The reporters require speciation events, but the event log cannot provide them.")
        } else if report_dispersal && !event_log.with_dispersal() {
            Err("The reporters require dispersal events, but the event log cannot provide them.")
        } else {
            Ok(())
        };

        match (valid, raw.mode) {
            (Ok(_), _) => Ok(Self {
                event_log,
                reporters,
            }),
            (Err(error), ReplayMode::WarnOnly) => {
                warn!("{}", error);

                Ok(Self {
                    event_log,
                    reporters,
                })
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
#[serde(rename = "Replay")]
struct ReplayArgsRaw {
    #[serde(alias = "log")]
    event_log: EventLogReplay,
    #[serde(default)]
    mode: ReplayMode,
    reporters: Vec<ReporterPluginLibrary>,
}

#[derive(Debug, Deserialize)]
#[serde(default)]
#[serde(deny_unknown_fields)]
struct Sample {
    percentage: ClosedUnitF64,
    origin: SampleOrigin,
}

impl Default for Sample {
    fn default() -> Self {
        Self {
            percentage: ClosedUnitF64::one(),
            origin: SampleOrigin::Habitat,
        }
    }
}

#[derive(Debug, Deserialize)]
enum SampleOrigin {
    Habitat,
    List(LineageFileLoader),
}

#[derive(Debug, Serialize)]
pub struct Pause {
    pub before: NonNegativeF64,
}

impl<'de> DeserializeState<'de, Partition> for Pause {
    fn deserialize_state<D: Deserializer<'de>>(
        partition: &mut Partition,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw = PauseRaw::deserialize(deserializer)?;

        if partition.size().get() > 1 {
            return Err(serde::de::Error::custom(
                "Parallel pausing is not yet supported.",
            ));
        }

        Ok(Pause { before: raw.before })
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "Pause")]
pub struct PauseRaw {
    pub before: NonNegativeF64,
}

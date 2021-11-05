use std::{fmt, marker::PhantomData, ops::Deref};

use necsim_core::cogs::{MathsCore, RngCore};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_state::DeserializeState;
use structopt::StructOpt;

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, Partition};

use necsim_impls_std::{
    event_log::replay::EventLogReplay, lineage_file::loader::LineageFileLoader,
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

pub mod parse;
pub mod ser;

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

#[allow(dead_code)]
pub struct Base32RngState<M: MathsCore, G: RngCore<M>> {
    state: Base32String,
    rng: G,
    marker: PhantomData<M>,
}

impl<M: MathsCore, G: RngCore<M>> Base32RngState<M, G> {
    #[must_use]
    #[allow(dead_code)]
    pub fn into(self) -> G {
        self.rng
    }
}

impl<M: MathsCore, G: RngCore<M>> fmt::Debug for Base32RngState<M, G> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.state.fmt(fmt)
    }
}

impl<M: MathsCore, G: RngCore<M>> Serialize for Base32RngState<M, G> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.state.serialize(serializer)
    }
}

impl<'de, M: MathsCore, G: RngCore<M>> Deserialize<'de> for Base32RngState<M, G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let state = Base32String::deserialize(deserializer)?;

        let rng = bincode::Options::deserialize(bincode::options(), &state)
            .map_err(|_| serde::de::Error::custom(format!("invalid RNG state {}", state)))?;

        Ok(Self {
            state,
            rng,
            marker: PhantomData::<M>,
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(bound = "")]
pub enum Rng<M: MathsCore, G: RngCore<M>> {
    Seed(u64),
    Sponge(Base32String),
    State(Base32RngState<M, G>),
}

impl<'de, M: MathsCore, G: RngCore<M>> Deserialize<'de> for Rng<M, G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = RngRaw::<M, G>::deserialize(deserializer)?;

        let rng = match raw {
            RngRaw::Entropy => {
                let mut entropy = G::Seed::default();

                getrandom::getrandom(entropy.as_mut()).map_err(serde::de::Error::custom)?;

                Self::Sponge(Base32String::new(entropy.as_mut()))
            },
            RngRaw::Seed(seed) => Self::Seed(seed),
            RngRaw::Sponge(sponge) => Self::Sponge(sponge),
            RngRaw::State(state) => Self::State(state),
            RngRaw::StateElseSponge(state) => {
                match bincode::Options::deserialize(bincode::options(), &state) {
                    Ok(rng) => Self::State(Base32RngState {
                        state,
                        rng,
                        marker: PhantomData::<M>,
                    }),
                    Err(_) => Self::Sponge(state),
                }
            },
        };

        Ok(rng)
    }
}

#[derive(Debug, Deserialize)]
#[serde(bound = "")]
#[serde(rename = "Rng")]
pub enum RngRaw<M: MathsCore, G: RngCore<M>> {
    Entropy,
    Seed(u64),
    Sponge(Base32String),
    State(Base32RngState<M, G>),
    StateElseSponge(Base32String),
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

    pub fn get_event_log_check(&self) -> (anyhow::Result<()>, anyhow::Result<()>) {
        match self {
            Self::Monolithic(_) => (Ok(()), Ok(())),
            #[cfg(feature = "necsim-partitioning-mpi")]
            Self::Mpi(_) => (
                Err(anyhow::anyhow!(
                    necsim_partitioning_mpi::MpiLocalPartitionError::MissingEventLog
                )),
                Ok(()),
            ),
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

#[derive(Serialize, Debug)]
#[serde(rename = "Replay")]
#[allow(clippy::module_name_repetitions)]
pub struct ReplayArgs {
    #[serde(rename = "log", alias = "event_log")]
    pub event_log: EventLogReplay,
    pub mode: ReplayMode,
    pub reporters: AnyReporterPluginVec,
}

impl<'de> Deserialize<'de> for ReplayArgs {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = ReplayArgsRaw::deserialize(deserializer)?;

        let event_log = raw.event_log;
        let mode = raw.mode;
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

        match (valid, mode) {
            (Ok(_), _) => Ok(Self {
                event_log,
                mode,
                reporters,
            }),
            (Err(error), ReplayMode::WarnOnly) => {
                warn!("{}", error);

                Ok(Self {
                    event_log,
                    mode,
                    reporters,
                })
            },
            (Err(error), ReplayMode::Strict) => Err(serde::de::Error::custom(error)),
        }
    }
}

#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
#[allow(clippy::module_name_repetitions)]
#[serde(deny_unknown_fields)]
pub enum ReplayMode {
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

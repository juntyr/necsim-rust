#![allow(clippy::empty_enum)]

use std::{
    collections::HashSet,
    convert::TryFrom,
    fmt,
    fs::{self, File, OpenOptions},
    marker::PhantomData,
    ops::Deref,
    path::PathBuf,
};

use fnv::FnvBuildHasher;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_state::DeserializeState;
use structopt::StructOpt;

use necsim_core::{
    cogs::{MathsCore, RngCore},
    lineage::Lineage,
};
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveF64};
use necsim_impls_std::{
    event_log::replay::EventLogReplay,
    lineage_file::{loader::LineageFileLoader, saver::LineageFileSaver},
};
use necsim_partitioning_core::partition::Partition;

use rustcoalescence_scenarios::{
    almost_infinite::AlmostInfiniteArguments,
    non_spatial::NonSpatialArguments,
    spatially_explicit::{
        SpatiallyExplicitTurnoverMapArguments, SpatiallyExplicitUniformTurnoverArguments,
    },
    spatially_implicit::SpatiallyImplicitArguments,
};

use rustcoalescence_algorithms::RestartFixUpStrategy;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
use rustcoalescence_algorithms::AlgorithmParamters;

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
    rng: G,
    marker: PhantomData<M>,
}

impl<M: MathsCore, G: RngCore<M>> From<G> for Base32RngState<M, G> {
    fn from(rng: G) -> Self {
        Self {
            rng,
            marker: PhantomData::<M>,
        }
    }
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
        match bincode::Options::serialize(bincode::options(), &self.rng) {
            Ok(state) => Base32String::new(&state).fmt(fmt),
            Err(_) => fmt.write_str("InvalidRngState"),
        }
    }
}

impl<M: MathsCore, G: RngCore<M>> Serialize for Base32RngState<M, G> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let state = bincode::Options::serialize(bincode::options(), &self.rng)
            .map_err(serde::ser::Error::custom)?;

        Base32String::new(&state).serialize(serializer)
    }
}

impl<'de, M: MathsCore, G: RngCore<M>> Deserialize<'de> for Base32RngState<M, G> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let state = Base32String::deserialize(deserializer)?;

        let rng = bincode::Options::deserialize(bincode::options(), &state)
            .map_err(|_| serde::de::Error::custom(format!("invalid RNG state {}", state)))?;

        Ok(Self {
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
    #[serde(alias = "Classical")]
    Gillespie(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::gillespie::GillespieAlgorithm as rustcoalescence_algorithms::AlgorithmParamters>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
    #[serde(alias = "SkippingGillespie")]
    EventSkipping(
        #[serde(deserialize_state)]
        <rustcoalescence_algorithms_gillespie::event_skipping::EventSkippingAlgorithm as AlgorithmParamters>::Arguments,
    ),
    #[cfg(feature = "rustcoalescence-algorithms-cuda")]
    #[serde(alias = "CUDA")]
    Cuda(#[serde(deserialize_state)] <rustcoalescence_algorithms_cuda::CudaAlgorithm as AlgorithmParamters>::Arguments),
    #[cfg(feature = "rustcoalescence-algorithms-independent")]
    Independent(
        #[serde(deserialize_state)] <rustcoalescence_algorithms_independent::IndependentAlgorithm as AlgorithmParamters>::Arguments,
    ),
}

impl Serialize for Algorithm {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        #[allow(unreachable_patterns, clippy::single_match_else)]
        match self {
            #[cfg(feature = "rustcoalescence-algorithms-gillespie")]
            Self::Gillespie(args) => {
                serializer.serialize_newtype_variant(stringify!(Algorithm), 0, "Gillespie", args)
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
    #[serde(alias = "SpatiallyExplicit")]
    SpatiallyExplicitUniformTurnover(SpatiallyExplicitUniformTurnoverArguments),
    SpatiallyExplicitTurnoverMap(SpatiallyExplicitTurnoverMapArguments),
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

#[derive(Debug, Serialize)]
pub struct Sample {
    pub percentage: ClosedUnitF64,
    pub origin: SampleOrigin,
    pub mode: SampleMode,
}

impl Default for Sample {
    fn default() -> Self {
        let raw = SampleRaw::default();

        Self {
            percentage: raw.percentage,
            origin: raw.origin,
            mode: raw.mode,
        }
    }
}

impl<'de> DeserializeState<'de, &'de Option<Pause>> for Sample {
    fn deserialize_state<D: Deserializer<'de>>(
        pause: &mut &'de Option<Pause>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw = SampleRaw::deserialize_state(pause, deserializer)?;

        match (&raw.origin, &raw.mode) {
            (SampleOrigin::Habitat, SampleMode::Genesis)
            | (
                SampleOrigin::List(_) | SampleOrigin::Bincode(_),
                SampleMode::Resume | SampleMode::FixUp(_) | SampleMode::Restart(_),
            ) => (),
            (
                SampleOrigin::Habitat,
                SampleMode::Resume | SampleMode::FixUp(_) | SampleMode::Restart(_),
            ) => {
                return Err(serde::de::Error::custom(
                    "`Habitat` origin is only compatible with `Genesis` mode",
                ));
            },
            (SampleOrigin::List(_) | SampleOrigin::Bincode(_), SampleMode::Genesis) => {
                return Err(serde::de::Error::custom(
                    "`Genesis` mode is only compatible with `Habitat` origin",
                ));
            },
        }

        let pre_resume_bound = match &raw.mode {
            SampleMode::Genesis | SampleMode::Resume => None,
            SampleMode::FixUp(_) => {
                if let Some(pause) = pause {
                    match pause.mode {
                        PauseMode::Resume => {
                            return Err(serde::de::Error::custom(
                                "`FixUp` sample mode is incompatible with `Resume` pause mode,\n \
                                 use `Restart` instead",
                            ))
                        },
                        PauseMode::FixUp | PauseMode::Restart => (),
                    }

                    match PositiveF64::new(pause.before.get()) {
                        Ok(fix_at) => Some(fix_at),
                        Err(_) => {
                            return Err(serde::de::Error::custom(
                                "`FixUp` mode cannot be used at simulation genesis time 0.0",
                            ))
                        },
                    }
                } else {
                    return Err(serde::de::Error::custom(
                        "`FixUp` mode requires an immediate pause to save the fixed individuals",
                    ));
                }
            },
            SampleMode::Restart(SampleModeRestart { after }) => {
                Some(PositiveF64::max_after(*after, *after))
            },
        };

        let lineages = match &raw.origin {
            SampleOrigin::Habitat => None,
            SampleOrigin::List(lineages) => Some(lineages.iter()),
            SampleOrigin::Bincode(loader) => Some(loader.get_lineages().iter()),
        };

        if let (Some(lineages), Some(pre_resume_bound)) = (lineages, pre_resume_bound) {
            for lineage in lineages {
                if lineage.last_event_time >= pre_resume_bound {
                    return Err(serde::de::Error::custom(format!(
                        "Lineage #{} at time {} is not before the resume point",
                        lineage.global_reference, lineage.last_event_time
                    )));
                }
            }
        }

        Ok(Self {
            percentage: raw.percentage,
            origin: raw.origin,
            mode: raw.mode,
        })
    }
}

#[derive(Debug, DeserializeState)]
#[serde(deserialize_state = "&'de Option<Pause>")]
#[serde(default)]
#[serde(deny_unknown_fields)]
#[serde(rename = "Sample")]
struct SampleRaw {
    percentage: ClosedUnitF64,
    #[serde(deserialize_state)]
    origin: SampleOrigin,
    mode: SampleMode,
}

impl Default for SampleRaw {
    fn default() -> Self {
        Self {
            percentage: ClosedUnitF64::one(),
            origin: SampleOrigin::Habitat,
            mode: SampleMode::Genesis,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SampleMode {
    Genesis,
    Resume,
    FixUp(RestartFixUpStrategy),
    Restart(SampleModeRestart),
}

impl Default for SampleMode {
    fn default() -> Self {
        Self::Genesis
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SampleModeRestart {
    pub after: NonNegativeF64,
}

#[derive(Debug, Serialize)]
pub struct Pause {
    pub before: NonNegativeF64,
    pub config: ResumeConfig,
    pub destiny: SampleDestiny,
    #[serde(default)]
    pub mode: PauseMode,
}

#[derive(Debug, Serialize)]
#[serde(rename = "Pause")]
pub struct FuturePause {
    pub before: NonNegativeF64,
    pub mode: PauseMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PauseMode {
    Resume,
    FixUp,
    Restart,
}

impl Default for PauseMode {
    fn default() -> Self {
        Self::Resume
    }
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

        if matches!(raw.mode, PauseMode::FixUp) && raw.before == NonNegativeF64::zero() {
            return Err(serde::de::Error::custom(
                "pause mode `FixUp` requires a positive non-zero pause time",
            ));
        }

        Ok(Pause {
            before: raw.before,
            config: raw.config,
            destiny: raw.destiny,
            mode: raw.mode,
        })
    }
}

#[derive(Serialize)]
pub enum SampleOrigin {
    Habitat,
    List(Vec<Lineage>),
    Bincode(LineageFileLoader),
}

impl fmt::Display for SampleOrigin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Habitat => fmt.write_str("Habitat"),
            Self::List(_) => fmt.write_str("List"),
            Self::Bincode(_) => fmt.write_str("Bincode"),
        }
    }
}

impl fmt::Display for SampleOriginRaw {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Habitat => fmt.write_str("Habitat"),
            Self::List(_) => fmt.write_str("List"),
            Self::Bincode(_) => fmt.write_str("Bincode"),
        }
    }
}

impl fmt::Debug for SampleOrigin {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        struct VecLineages(usize);

        impl fmt::Debug for VecLineages {
            fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
                write!(fmt, "Vec<Lineage; {}>", self.0)
            }
        }

        match self {
            Self::Habitat => fmt.debug_struct(stringify!(Habitat)).finish(),
            Self::List(lineages) => fmt
                .debug_tuple(stringify!(List))
                .field(&VecLineages(lineages.len()))
                .finish(),
            Self::Bincode(loader) => fmt
                .debug_tuple(stringify!(Bincode))
                .field(&VecLineages(loader.get_lineages().len()))
                .finish(),
        }
    }
}

impl<'de> DeserializeState<'de, &'de Option<Pause>> for SampleOrigin {
    fn deserialize_state<D: Deserializer<'de>>(
        pause: &mut &'de Option<Pause>,
        deserializer: D,
    ) -> Result<Self, D::Error> {
        let raw = SampleOriginRaw::deserialize(deserializer)?;

        if let Some(pause) = pause {
            if matches!(pause.destiny, SampleDestiny::List)
                && !matches!(raw, SampleOriginRaw::List(_))
            {
                return Err(serde::de::Error::custom(format!(
                    "`List` pause destiny requires `List` origin sample, found `{}`",
                    raw
                )));
            }
        }

        let lineages = match &raw {
            SampleOriginRaw::Habitat => return Ok(Self::Habitat),
            SampleOriginRaw::List(lineages) => lineages.iter(),
            SampleOriginRaw::Bincode(loader) => loader.get_lineages().iter(),
        };

        let mut global_references =
            HashSet::with_capacity_and_hasher(lineages.len(), FnvBuildHasher::default());

        for lineage in lineages {
            if !global_references.insert(lineage.global_reference.clone()) {
                return Err(serde::de::Error::custom(format!(
                    "duplicate lineage reference #{}",
                    lineage.global_reference
                )));
            }
        }

        match raw {
            SampleOriginRaw::Habitat => Ok(Self::Habitat),
            SampleOriginRaw::List(lineages) => Ok(Self::List(lineages)),
            SampleOriginRaw::Bincode(loader) => Ok(Self::Bincode(loader)),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "Pause")]
pub struct PauseRaw {
    pub before: NonNegativeF64,
    pub config: ResumeConfig,
    pub destiny: SampleDestiny,
    pub mode: PauseMode,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SampleDestiny {
    List,
    Bincode(LineageFileSaver),
}

#[derive(Deserialize)]
#[serde(try_from = "PathBuf")]
pub struct ResumeConfig {
    file: File,
    path: PathBuf,
    temp: bool,
}

impl Drop for ResumeConfig {
    fn drop(&mut self) {
        if self.temp {
            std::mem::drop(fs::remove_file(self.path.clone()));
        }
    }
}

impl fmt::Debug for ResumeConfig {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        self.path.fmt(fmt)
    }
}

impl Serialize for ResumeConfig {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.path.serialize(serializer)
    }
}

impl TryFrom<PathBuf> for ResumeConfig {
    type Error = anyhow::Error;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let file = OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&path)?;

        Ok(Self {
            file,
            path,
            temp: true,
        })
    }
}

impl ResumeConfig {
    pub fn write(mut self, config: &str) -> anyhow::Result<()> {
        std::io::Write::write_fmt(&mut self.file, format_args!("{}\n", config))?;

        self.temp = false;

        Ok(())
    }
}

#[derive(Debug, Deserialize)]
enum SampleOriginRaw {
    Habitat,
    List(Vec<Lineage>),
    Bincode(LineageFileLoader),
}

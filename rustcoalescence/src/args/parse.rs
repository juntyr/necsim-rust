use anyhow::{Context, Result};
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::{de::IgnoredAny, Deserialize, Deserializer};
use serde_state::DeserializeState;

use necsim_core_bond::{ClosedUnitF64, Partition, PositiveUnitF64};
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use necsim_plugins_core::import::AnyReporterPluginVec;

use super::{Algorithm, CommandArgs, CommonArgs, Partitioning, Pause, Rng, Scenario, SimulateArgs};

impl SimulateArgs {
    pub fn try_parse(command_args: CommandArgs) -> Result<Self> {
        let ron_args = into_ron_str(command_args);

        // Check for the overall config stucture
        //  (1) are all required fields defined
        //  (2) are any unknown fields defined
        let SimulateArgsFields { .. } = try_parse("simulate", &ron_args)?;

        let SimulateArgsPartitioningOnly { partitioning } = try_parse("simulate", &ron_args)?;
        let mut partition = partitioning.get_partition();

        // Only log to stdout/stderr if the partition is the root partition
        log::set_max_level(if partitioning.is_root() {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Off
        });

        let mut event_log_check = partitioning.get_event_log_check();

        let SimulateArgsEventLogOnly { event_log } =
            try_parse_state("simulate", &ron_args, &mut event_log_check)?;

        match &event_log {
            None => event_log_check.0,
            Some(_) => event_log_check.1,
        }
        .map_err(|err| anyhow::anyhow!("simulate.*: {}", err))
        .context("Failed to parse the simulate subcommand arguments.")?;

        let SimulateArgsCommon {
            speciation_probability_per_generation,
            sample_percentage,
            scenario,
            reporters,
        } = try_parse("simulate", &ron_args)?;

        let SimulateArgsStatePartition { algorithm, pause } =
            try_parse_state("simulate", &ron_args, &mut partition)?;

        let SimulateArgsRngOnly { rng } = try_parse("simulate", &ron_args)?;

        // TODO: Transform the RNG based on scenario + algorithms

        let args = SimulateArgs {
            common: CommonArgs {
                speciation_probability_per_generation,
                sample_percentage,
                rng,
            },
            scenario,
            algorithm,
            partitioning,
            event_log,
            reporters,
            pause,
        };

        Ok(args)
    }
}

/// Transform the `command_args` into a RON `String`
pub fn into_ron_str(command_args: CommandArgs) -> String {
    let mut ron_args = String::new();

    for arg in command_args.args {
        ron_args.push_str(&arg);
        ron_args.push(' ');
    }

    let ron_args_trimmed = ron_args.trim();

    let mut ron_args =
        String::from("#![enable(unwrap_variant_newtypes, unwrap_newtypes, implicit_some)]");
    ron_args.reserve(ron_args_trimmed.len());

    if !ron_args_trimmed.starts_with('(') {
        ron_args.push('(');
    }
    ron_args.push_str(ron_args_trimmed);
    if !ron_args_trimmed.starts_with('(') {
        ron_args.push(')');
    }

    ron_args
}

pub fn ron_config() -> PrettyConfig {
    PrettyConfig::default()
        .decimal_floats(true)
        .struct_names(true)
        .extensions(
            Extensions::UNWRAP_VARIANT_NEWTYPES
                | Extensions::UNWRAP_NEWTYPES
                | Extensions::IMPLICIT_SOME,
        )
        .output_extensions(false)
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
#[serde(rename = "Simulate")]
#[allow(dead_code)]
struct SimulateArgsFields {
    #[serde(alias = "speciation_probability_per_generation")]
    speciation: IgnoredAny,

    #[serde(alias = "sample_percentage")]
    sample: IgnoredAny,

    #[serde(alias = "randomness")]
    #[serde(default)]
    rng: IgnoredAny,

    scenario: IgnoredAny,

    algorithm: IgnoredAny,

    #[serde(default)]
    partitioning: IgnoredAny,

    #[serde(alias = "event_log")]
    #[serde(default)]
    log: Option<IgnoredAny>,

    reporters: Vec<IgnoredAny>,

    #[serde(default)]
    pause: Option<IgnoredAny>,
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsPartitioningOnly {
    #[serde(default)]
    partitioning: Partitioning,
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "(anyhow::Result<()>, anyhow::Result<()>)")]
#[serde(rename = "Simulate")]
struct SimulateArgsEventLogOnly {
    #[serde(alias = "log")]
    #[serde(default)]
    #[serde(deserialize_state_with = "deserialize_state_event_log")]
    event_log: Option<EventLogRecorder>,
}

fn deserialize_state_event_log<'de, D: Deserializer<'de>>(
    event_log_check: &mut (anyhow::Result<()>, anyhow::Result<()>),
    deserializer: D,
) -> Result<Option<EventLogRecorder>, D::Error> {
    let maybe_event_log = <Option<EventLogRecorder>>::deserialize(deserializer)?;

    if maybe_event_log.is_none() {
        event_log_check
            .0
            .as_ref()
            .map_err(serde::de::Error::custom)?;
    } else {
        event_log_check
            .1
            .as_ref()
            .map_err(serde::de::Error::custom)?;
    }

    Ok(maybe_event_log)
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsCommon {
    #[serde(alias = "speciation")]
    speciation_probability_per_generation: PositiveUnitF64,

    #[serde(alias = "sample")]
    sample_percentage: ClosedUnitF64,

    scenario: Scenario,

    reporters: AnyReporterPluginVec,
}

#[derive(DeserializeState)]
#[serde(deserialize_state = "Partition")]
#[serde(rename = "Simulate")]
struct SimulateArgsStatePartition {
    #[serde(deserialize_state)]
    algorithm: Algorithm,

    #[serde(default)]
    #[serde(deserialize_state)]
    pause: Option<Pause>,
}

#[derive(Deserialize)]
#[serde(rename = "Simulate")]
struct SimulateArgsRngOnly {
    #[serde(alias = "randomness")]
    #[serde(default)]
    rng: Rng,
}

#[allow(clippy::module_name_repetitions)]
pub fn try_parse<'de, D: Deserialize<'de>>(subcommand: &str, ron_args: &'de str) -> Result<D> {
    let mut de_ron = ron::Deserializer::from_str(ron_args).with_context(|| {
        format!(
            "Failed to create the {} subcommand argument parser.",
            subcommand
        )
    })?;

    let mut track = serde_path_to_error::Track::new();
    let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

    match D::deserialize(de) {
        Ok(args) => Ok(args),
        Err(err) => {
            let path = track.path();

            Err(anyhow::anyhow!(
                "{}{}{}{}: {}",
                subcommand,
                if path.iter().count() >= 1 { "." } else { "" },
                path,
                if path.iter().count() >= 1 { "" } else { "*" },
                err,
            ))
        },
    }
    .with_context(|| format!("Failed to parse the {} subcommand arguments.", subcommand))
}

pub fn try_parse_state<'de, D: DeserializeState<'de, Seed>, Seed: ?Sized>(
    subcommand: &str,
    ron_args: &'de str,
    seed: &'de mut Seed,
) -> Result<D> {
    let mut de_ron = ron::Deserializer::from_str(ron_args).with_context(|| {
        format!(
            "Failed to create the {} subcommand argument parser.",
            subcommand
        )
    })?;

    let mut track = serde_path_to_error::Track::new();
    let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

    match D::deserialize_state(seed, de) {
        Ok(args) => Ok(args),
        Err(err) => {
            let path = track.path();

            Err(anyhow::anyhow!(
                "{}{}{}{}: {}",
                subcommand,
                if path.iter().count() >= 1 { "." } else { "" },
                path,
                if path.iter().count() >= 1 { "" } else { "*" },
                err,
            ))
        },
    }
    .with_context(|| format!("Failed to parse the {} subcommand arguments.", subcommand))
}

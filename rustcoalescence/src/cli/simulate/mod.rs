use anyhow::Context;
use log::LevelFilter;
use necsim_core::{
    cogs::{MathsCore, RngCore},
    lineage::Lineage,
};
use necsim_impls_std::event_log::recorder::EventLogRecorder;
use serde::{de::IgnoredAny, Deserialize, Deserializer, Serialize};

use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, Partition, PositiveUnitF64};
use necsim_partitioning_core::Partitioning as _;
use necsim_plugins_core::{import::AnyReporterPluginVec, match_any_reporter_plugin_vec};

use crate::{
    args::{
        parse::{into_ron_str, ron_config, try_parse, try_parse_state},
        ser::{BufferingSerialize, BufferingSerializer},
        Algorithm, Base32String, CommandArgs, Partitioning, Pause, Rng, SampleDestiny, Scenario,
    },
    reporter::DynamicReporterContext,
};

#[macro_use]
mod r#impl;

#[cfg(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
))]
mod dispatch;

#[allow(clippy::module_name_repetitions, clippy::too_many_lines)]
pub fn simulate_with_logger(simulate_args: CommandArgs) -> anyhow::Result<()> {
    log::set_max_level(LevelFilter::Info);

    let ron_args = into_ron_str(simulate_args);

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

    let partial_simulate_args = BufferingPartialSimulateArgs::new(
        speciation_probability_per_generation,
        sample_percentage,
        &scenario,
        &algorithm,
        &partitioning,
        &event_log,
        &reporters,
        &pause,
    )?;

    let partial_resume_args = BufferingPartialResumeArgs::new(
        speciation_probability_per_generation,
        &scenario,
        &algorithm,
        &partitioning,
        &event_log,
        &reporters,
    )?;

    let pause_before = pause.as_ref().map(|pause| pause.before);

    let result = match_any_reporter_plugin_vec!(reporters => |reporter| {
        use necsim_partitioning_monolithic::MonolithicLocalPartition;
        #[cfg(feature = "necsim-partitioning-mpi")]
        use necsim_partitioning_mpi::MpiLocalPartition;

        // Initialise the local partition and the simulation
        match partitioning {
            Partitioning::Monolithic(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), event_log
            ).with_context(|| "Failed to initialise the local monolithic partition.")? {
                MonolithicLocalPartition::Live(partition) => dispatch::simulate_with_logger(
                    *partition, speciation_probability_per_generation, sample_percentage,
                    scenario, algorithm, pause_before, &ron_args, partial_simulate_args,
                ),
                MonolithicLocalPartition::Recorded(partition) => dispatch::simulate_with_logger(
                    *partition, speciation_probability_per_generation, sample_percentage,
                    scenario, algorithm, pause_before, &ron_args, partial_simulate_args,
                ),
            },
            #[cfg(feature = "necsim-partitioning-mpi")]
            Partitioning::Mpi(partitioning) => match partitioning.into_local_partition(
                DynamicReporterContext::new(reporter), event_log
            ).with_context(|| "Failed to initialise the local MPI partition.")? {
                MpiLocalPartition::Root(partition) => dispatch::simulate_with_logger(
                    *partition, speciation_probability_per_generation, sample_percentage,
                    scenario, algorithm, pause_before, &ron_args, partial_simulate_args,
                ),
                MpiLocalPartition::Parallel(partition) => dispatch::simulate_with_logger(
                    *partition, speciation_probability_per_generation, sample_percentage,
                    scenario, algorithm, pause_before, &ron_args, partial_simulate_args,
                ),
            },
        }
    })?;

    match &result {
        SimulationResult::Done { time, steps } => info!(
            "The simulation finished at time {} after {} steps.\n",
            time.get(),
            steps
        ),
        SimulationResult::Paused { time, steps, .. } => info!(
            "The simulation paused at time {} after {} steps.\n",
            time.get(),
            steps
        ),
    }

    if let (Some(pause), SimulationResult::Paused { lineages, rng, .. }) = (pause, result) {
        match pause.destiny {
            // TODO: Adapt the config sample
            SampleDestiny::List => (),
            SampleDestiny::Serde(lineage_file) => lineage_file
                .write(lineages.iter())
                .context("Failed to write the remaining lineages.")?,
        };

        let resume_args = ResumeArgs::new(partial_resume_args, &rng)?;

        let resume_str = ron::ser::to_string_pretty(&resume_args, ron_config())
            .context("Failed to generate config to resume the simulation.")?;

        pause
            .config
            .write(resume_str.trim_start_matches("Simulate"))
            .context("Failed to write the config to resume the simulation.")?;
    }

    Ok(())
}

#[cfg(not(any(
    feature = "rustcoalescence-algorithms-gillespie",
    feature = "rustcoalescence-algorithms-independent",
    feature = "rustcoalescence-algorithms-cuda"
)))]
mod dispatch {
    use necsim_core::reporter::Reporter;
    use necsim_core_bond::{ClosedUnitF64, NonNegativeF64, PositiveUnitF64};
    use necsim_partitioning_core::LocalPartition;

    use crate::args::{Algorithm as AlgorithmArgs, Scenario as ScenarioArgs};

    use super::{BufferingPartialSimulateArgs, SimulationResult};

    #[allow(clippy::too_many_arguments, clippy::needless_pass_by_value)]
    pub(super) fn simulate_with_logger<R: Reporter, P: LocalPartition<R>>(
        _local_partition: P,
        _speciation_probability_per_generation: PositiveUnitF64,
        _sample_percentage: ClosedUnitF64,
        _scenario: ScenarioArgs,
        _algorithm: AlgorithmArgs,
        _pause_before: Option<NonNegativeF64>,
        _ron_args: &str,
        _partial_simulate_args: BufferingPartialSimulateArgs,
    ) -> anyhow::Result<SimulationResult> {
        anyhow::bail!("rustcoalescence must be compiled to support at least one algorithm.")
    }
}

#[allow(dead_code)]
struct BufferingPartialSimulateArgs {
    speciation: BufferingSerialize,
    sample: BufferingSerialize,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
    pause: BufferingSerialize,
}

impl BufferingPartialSimulateArgs {
    #[allow(clippy::too_many_arguments)]
    fn new(
        speciation: PositiveUnitF64,
        sample: ClosedUnitF64,
        scenario: &Scenario,
        algorithm: &Algorithm,
        partitioning: &Partitioning,
        event_log: &Option<EventLogRecorder>,
        reporters: &AnyReporterPluginVec,
        pause: &Option<Pause>,
    ) -> anyhow::Result<Self> {
        (|| -> anyhow::Result<Self> {
            Ok(Self {
                speciation: speciation.serialize(BufferingSerializer)?,
                sample: sample.serialize(BufferingSerializer)?,
                scenario: scenario.serialize(BufferingSerializer)?,
                algorithm: algorithm.serialize(BufferingSerializer)?,
                partitioning: partitioning.serialize(BufferingSerializer)?,
                log: event_log.serialize(BufferingSerializer)?,
                reporters: reporters.serialize(BufferingSerializer)?,
                pause: pause.serialize(BufferingSerializer)?,
            })
        })()
        .context("Failed to normalise the simulation config.")
    }
}

#[derive(Serialize)]
#[serde(rename = "Simulate")]
struct BufferingSimulateArgs {
    speciation: BufferingSerialize,
    sample: BufferingSerialize,
    rng: BufferingSerialize,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
    pause: BufferingSerialize,
}

impl BufferingSimulateArgs {
    #[allow(dead_code)]
    fn new<M: MathsCore, G: RngCore<M>>(
        partial: BufferingPartialSimulateArgs,
        rng: &Rng<M, G>,
    ) -> anyhow::Result<Self> {
        (|| -> anyhow::Result<Self> {
            Ok(Self {
                speciation: partial.speciation,
                sample: partial.sample,
                rng: rng.serialize(BufferingSerializer)?,
                scenario: partial.scenario,
                algorithm: partial.algorithm,
                partitioning: partial.partitioning,
                log: partial.log,
                reporters: partial.reporters,
                pause: partial.pause,
            })
        })()
        .context("Failed to normalise the simulation config.")
    }
}

struct BufferingPartialResumeArgs {
    speciation: BufferingSerialize,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
}

impl BufferingPartialResumeArgs {
    fn new(
        speciation: PositiveUnitF64,
        scenario: &Scenario,
        algorithm: &Algorithm,
        partitioning: &Partitioning,
        event_log: &Option<EventLogRecorder>,
        reporters: &AnyReporterPluginVec,
    ) -> anyhow::Result<Self> {
        (|| -> anyhow::Result<Self> {
            Ok(Self {
                speciation: speciation.serialize(BufferingSerializer)?,
                scenario: scenario.serialize(BufferingSerializer)?,
                algorithm: algorithm.serialize(BufferingSerializer)?,
                partitioning: partitioning.serialize(BufferingSerializer)?,
                log: event_log.serialize(BufferingSerializer)?,
                reporters: reporters.serialize(BufferingSerializer)?,
            })
        })()
        .context("Failed to generate config to resume the simulation.")
    }
}

#[derive(Serialize)]
#[serde(rename = "Simulate")]
struct ResumeArgs {
    speciation: BufferingSerialize,
    sample: ClosedUnitF64,
    rng: BufferingSerialize,
    scenario: BufferingSerialize,
    algorithm: BufferingSerialize,
    partitioning: BufferingSerialize,
    log: BufferingSerialize,
    reporters: BufferingSerialize,
    pause: Option<Pause>,
}

impl ResumeArgs {
    fn new(partial: BufferingPartialResumeArgs, rng: &ResumingRng) -> anyhow::Result<Self> {
        (|| -> anyhow::Result<Self> {
            Ok(Self {
                speciation: partial.speciation,
                sample: ClosedUnitF64::one(),
                rng: rng.serialize(BufferingSerializer)?,
                scenario: partial.scenario,
                algorithm: partial.algorithm,
                partitioning: partial.partitioning,
                log: partial.log,
                reporters: partial.reporters,
                pause: None,
            })
        })()
        .context("Failed to generate config to resume the simulation.")
    }
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

#[allow(dead_code)]
enum SimulationResult {
    Done {
        time: NonNegativeF64,
        steps: u64,
    },
    Paused {
        time: NonNegativeF64,
        steps: u64,
        lineages: Vec<Lineage>,
        rng: ResumingRng,
    },
}

#[derive(Debug, Serialize)]
#[serde(rename = "Rng")]
#[allow(dead_code)]
enum ResumingRng {
    State(Base32String),
}

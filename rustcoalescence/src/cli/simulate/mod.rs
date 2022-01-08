use anyhow::Context;
use derive_builder::Builder;
use log::LevelFilter;
use necsim_impls_std::lineage_file::loader::LineageFileLoader;
use serde::Serialize;

use necsim_core::lineage::Lineage;
use necsim_core_bond::{ClosedUnitF64, NonNegativeF64};

use crate::args::{
    parse::{into_ron_str, try_print},
    ser::BufferingSerializeResult,
    CommandArgs, Pause, Sample, SampleDestiny, SampleMode, SampleOrigin,
};

mod dispatch;
mod launch;
mod parse;

use dispatch::dispatch;

#[allow(clippy::module_name_repetitions)]
pub fn simulate_with_logger(simulate_args: CommandArgs) -> anyhow::Result<()> {
    log::set_max_level(LevelFilter::Info);

    let ron_args = into_ron_str(simulate_args);
    parse::fields::parse_and_normalise(&ron_args)?;
    let mut normalised_args = BufferingSimulateArgs::builder();

    let partitioning = parse::partitioning::parse_and_normalise(&ron_args, &mut normalised_args)?;

    // Only log to stdout/stderr if the partition is the root partition
    log::set_max_level(if partitioning.is_root() {
        log::LevelFilter::Info
    } else {
        log::LevelFilter::Off
    });

    let sample = parse::sample::parse_and_normalise(&ron_args, &mut normalised_args)?;
    let pause =
        parse::pause::parse_and_normalise(&ron_args, &mut normalised_args, &partitioning, &sample)?;

    let speciation_probability_per_generation =
        parse::speciation::parse_and_normalise(&ron_args, &mut normalised_args)?;

    let scenario = parse::scenario::parse_and_normalise(&ron_args, &mut normalised_args)?;
    let algorithm =
        parse::algorithm::parse_and_normalise(&ron_args, &mut normalised_args, &partitioning)?;

    let event_log = parse::event_log::parse_and_normalise(
        &ron_args,
        &mut normalised_args,
        &partitioning,
        &sample,
        &pause,
    )?;

    let reporters = parse::reporters::parse_and_normalise(&ron_args, &mut normalised_args)?;

    let result = dispatch(
        partitioning,
        event_log,
        reporters,
        speciation_probability_per_generation,
        sample,
        scenario,
        algorithm,
        pause.as_ref().map(|pause| pause.before),
        &ron_args,
        &mut normalised_args,
    )?;

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

    if let (Some(pause), SimulationResult::Paused { lineages, .. }) = (pause, result) {
        let resume_str = normalised_args
            .sample(&Sample {
                percentage: ClosedUnitF64::one(),
                origin: match pause.destiny {
                    SampleDestiny::List => SampleOrigin::List(lineages),
                    SampleDestiny::Bincode(lineage_file) => {
                        let path = lineage_file.path().to_owned();

                        lineage_file
                            .write(lineages.iter())
                            .context("Failed to write the remaining lineages.")?;

                        SampleOrigin::Bincode(
                            LineageFileLoader::try_new(&path)
                                .context("Failed to write the remaining lineages.")?,
                        )
                    },
                },
                mode: SampleMode::Resume,
            })
            .pause(&Option::<Pause>::None)
            .build()
            .map_err(anyhow::Error::new)
            .and_then(|resume_args| try_print(&resume_args))
            .context("Failed to generate the config to resume the simulation.")?;

        pause
            .config
            .write(resume_str.trim_start_matches("Simulate"))
            .context("Failed to write the config to resume the simulation.")?;
    }

    Ok(())
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
    },
}

#[derive(Serialize, Builder)]
#[builder(setter(into))]
#[serde(rename = "Simulate")]
struct BufferingSimulateArgs {
    speciation: BufferingSerializeResult,
    sample: BufferingSerializeResult,
    rng: BufferingSerializeResult,
    scenario: BufferingSerializeResult,
    algorithm: BufferingSerializeResult,
    partitioning: BufferingSerializeResult,
    log: BufferingSerializeResult,
    reporters: BufferingSerializeResult,
    pause: BufferingSerializeResult,
}

impl BufferingSimulateArgs {
    #[must_use]
    pub fn builder() -> BufferingSimulateArgsBuilder {
        BufferingSimulateArgsBuilder::default()
    }
}

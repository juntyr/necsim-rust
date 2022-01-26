use anyhow::{Context, Result};

use necsim_core::lineage::Lineage;
use necsim_core_bond::ClosedUnitF64;

use necsim_impls_std::lineage_file::loader::LineageFileLoader;

use rustcoalescence_algorithms::RestartFixUpStrategy;

use crate::args::{
    parse::try_print, FuturePause, Pause, PauseMode, Sample, SampleDestiny, SampleMode,
    SampleModeRestart, SampleOrigin,
};

use super::BufferingSimulateArgsBuilder;

pub(super) fn write_resume_config(
    mut normalised_args: BufferingSimulateArgsBuilder,
    pause: Pause,
    lineages: Vec<Lineage>,
) -> Result<()> {
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
            mode: match pause.mode {
                PauseMode::Resume => SampleMode::Resume,
                PauseMode::FixUp => SampleMode::FixUp(RestartFixUpStrategy::default()),
                PauseMode::Restart => SampleMode::Restart(SampleModeRestart {
                    after: pause.before,
                }),
            },
        })
        .pause(&match pause.mode {
            PauseMode::Resume | PauseMode::Restart => None,
            PauseMode::FixUp => Some(FuturePause {
                before: pause.before,
                mode: PauseMode::Restart,
            }),
        })
        .build()
        .map_err(anyhow::Error::new)
        .and_then(|resume_args| try_print(&resume_args))
        .context("Failed to generate the config to resume the simulation.")?;

    pause
        .config
        .write(resume_str.trim_start_matches("Simulate"))
        .context("Failed to write the config to resume the simulation.")
}

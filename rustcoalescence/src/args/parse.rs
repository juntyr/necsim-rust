use anyhow::{Context, Result};
use serde::Deserialize;
use serde_state::DeserializeState;

use super::{CommandArgs, Partitioning, ReplayArgs, SimulateArgs};

/// Transform the `command_args` into a RON `String`
fn into_ron_args(command_args: CommandArgs) -> String {
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

#[derive(Deserialize)]
struct PartitioningOnly {
    #[serde(default)]
    partitioning: Partitioning,
}

impl SimulateArgs {
    pub fn try_parse(command_args: CommandArgs) -> Result<Self> {
        let ron_args = into_ron_args(command_args);
        let mut de_ron = ron::Deserializer::from_str(&ron_args)
            .context("Failed to create the simulate subcommand argument parser.")?;

        let mut track = serde_path_to_error::Track::new();
        let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

        let PartitioningOnly { partitioning } = match PartitioningOnly::deserialize(de) {
            Ok(args) => Ok(args),
            Err(err) => {
                let path = track.path();

                Err(anyhow::Error::msg(format!(
                    "simulate{}{}{}: {}",
                    if path.iter().count() >= 1 { "." } else { "" },
                    path,
                    if path.iter().count() >= 1 { "" } else { "*" },
                    err,
                )))
            },
        }
        .context("Failed to parse the simulate subcommand arguments.")?;

        let mut partition = partitioning.get_partition();

        // Only log to stdout/stderr if the partition is the root partition
        log::set_max_level(if partitioning.is_root() {
            log::LevelFilter::Info
        } else {
            log::LevelFilter::Off
        });

        let mut de_ron = ron::Deserializer::from_str(&ron_args)
            .context("Failed to create the simulate subcommand argument parser.")?;

        let mut track = serde_path_to_error::Track::new();
        let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

        let args = match SimulateArgs::deserialize_state(&mut partition, de) {
            Ok(args) => Ok(args),
            Err(err) => {
                let path = track.path();

                Err(anyhow::Error::msg(format!(
                    "simulate{}{}{}: {}",
                    if path.iter().count() >= 1 { "." } else { "" },
                    path,
                    if path.iter().count() >= 1 { "" } else { "*" },
                    err,
                )))
            },
        }
        .context("Failed to parse the simulate subcommand arguments.")?;

        Ok(args)
    }
}

impl ReplayArgs {
    pub fn try_parse(command_args: CommandArgs) -> Result<Self> {
        let ron_args = into_ron_args(command_args);
        let mut de_ron = ron::Deserializer::from_str(&ron_args)
            .context("Failed to create the replay subcommand argument parser.")?;

        let mut track = serde_path_to_error::Track::new();
        let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

        let args = match ReplayArgs::deserialize(de) {
            Ok(args) => Ok(args),
            Err(err) => {
                let path = track.path();

                Err(anyhow::Error::msg(format!(
                    "replay{}{}{}: {}",
                    if path.iter().count() >= 1 { "." } else { "" },
                    path,
                    if path.iter().count() >= 1 { "" } else { "*" },
                    err,
                )))
            },
        }
        .context("Failed to parse the replay subcommand arguments.")?;

        Ok(args)
    }
}

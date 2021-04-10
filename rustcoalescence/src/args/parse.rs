use anyhow::{Context, Result};
use serde::Deserialize;
use serde_state::DeserializeState;

use necsim_impls_no_std::partitioning::Partitioning;
use necsim_impls_std::bounded::Partition;

use super::{CommandArgs, ReplayArgs, SimulateArgs};

/// Transform the `command_args` into a RON `String`
fn into_ron_args(command_args: CommandArgs) -> String {
    let mut ron_args = String::new();

    for arg in command_args.args {
        ron_args.push_str(&arg);
        ron_args.push(' ');
    }

    let ron_args_trimmed = ron_args.trim();

    let mut ron_args = String::from("#![enable(unwrap_newtypes)] #![enable(implicit_some)]");
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

impl SimulateArgs {
    pub fn try_parse<P: Partitioning>(command_args: CommandArgs, partitioning: &P) -> Result<Self> {
        // Parse and validate all command line arguments for the simulate subcommand
        let ron_args = into_ron_args(command_args);

        let mut de_ron = ron::Deserializer::from_str(&ron_args)
            .context("Failed to create the simulate subcommand argument parser.")?;

        let mut track = serde_path_to_error::Track::new();
        let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

        let mut partition = Partition::try_new(
            partitioning.get_rank(),
            partitioning.get_number_of_partitions(),
        )?;

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
        // Parse and validate all command line arguments for the replay subcommand
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

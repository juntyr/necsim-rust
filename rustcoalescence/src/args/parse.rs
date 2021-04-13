use anyhow::{Context, Result};
use serde_state::DeserializeState;

use necsim_impls_no_std::{bounded::Partition, partitioning::Partitioning};

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

fn try_parse_subcommand_arguments<'de, A: DeserializeState<'de, Partition>, P: Partitioning>(
    subcommand: &str,
    ron_args: &'de str,
    partitioning: &P,
) -> Result<A> {
    let mut de_ron = ron::Deserializer::from_str(ron_args).context(format!(
        "Failed to create the {} subcommand argument parser.",
        subcommand
    ))?;

    let mut track = serde_path_to_error::Track::new();
    let de = serde_path_to_error::Deserializer::new(&mut de_ron, &mut track);

    let mut partition = Partition::try_new(
        partitioning.get_rank(),
        partitioning.get_number_of_partitions(),
    )
    .map_err(anyhow::Error::msg)?;

    let args = match A::deserialize_state(&mut partition, de) {
        Ok(args) => Ok(args),
        Err(err) => {
            let path = track.path();

            Err(anyhow::Error::msg(format!(
                "{}{}{}{}: {}",
                subcommand,
                if path.iter().count() >= 1 { "." } else { "" },
                path,
                if path.iter().count() >= 1 { "" } else { "*" },
                err,
            )))
        },
    }
    .context(format!(
        "Failed to parse the {} subcommand arguments.",
        subcommand
    ))?;

    Ok(args)
}

impl SimulateArgs {
    pub fn try_parse<P: Partitioning>(command_args: CommandArgs, partitioning: &P) -> Result<Self> {
        // Parse and validate all command line arguments for a subcommand
        try_parse_subcommand_arguments("simulate", &into_ron_args(command_args), partitioning)
    }
}

impl ReplayArgs {
    pub fn try_parse<P: Partitioning>(command_args: CommandArgs, partitioning: &P) -> Result<Self> {
        // Parse and validate all command line arguments for a subcommand
        try_parse_subcommand_arguments("replay", &into_ron_args(command_args), partitioning)
    }
}

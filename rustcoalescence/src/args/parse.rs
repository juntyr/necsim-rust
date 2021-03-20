use anyhow::{Context, Result};
use serde_state::DeserializeState;

use necsim_impls_no_std::partitioning::Partitioning;
use necsim_impls_std::bounded::Partition;

use super::{SimulateArgs, SimulateCommandArgs};

impl SimulateArgs {
    pub fn try_parse<P: Partitioning>(
        command_args: SimulateCommandArgs,
        partitioning: &P,
    ) -> Result<Self> {
        // Parse and validate all command line arguments for the simulate subcommand
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

        let mut de_ron = ron::Deserializer::from_str(&ron_args)
            .context("Failed to parse the simulate arguments.")?;

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
        .context("Failed to parse the simulate arguments.")?;

        Ok(args)
    }
}

use std::convert::TryFrom;

use anyhow::Context;

use super::{SimulateArgs, SimulateCommandArgs};

impl TryFrom<SimulateCommandArgs> for SimulateArgs {
    type Error = anyhow::Error;

    fn try_from(command_args: SimulateCommandArgs) -> Result<Self, Self::Error> {
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

        let mut de = ron::Deserializer::from_str(&ron_args)
            .context("Failed to parse the simulate arguments.")?;

        let args: SimulateArgs = serde_path_to_error::deserialize(&mut de)
            .map_err(|err| {
                anyhow::Error::msg(format!(
                    "simulate{}{}{}: {}",
                    if err.path().iter().count() >= 1 {
                        "."
                    } else {
                        ""
                    },
                    err.path(),
                    if err.path().iter().count() >= 1 {
                        ""
                    } else {
                        "*"
                    },
                    err.inner(),
                ))
            })
            .context("Failed to parse the simulate arguments.")?;

        Ok(args)
    }
}

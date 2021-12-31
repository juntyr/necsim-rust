use anyhow::{Context, Result};
use ron::{extensions::Extensions, ser::PrettyConfig, Options};
use serde::{Deserialize, Serialize};
use serde_state::DeserializeState;

use super::CommandArgs;

/// Transform the `command_args` into a RON `String`
pub fn into_ron_str(command_args: CommandArgs) -> String {
    let mut ron_args = String::new();

    for arg in command_args.args {
        ron_args.push_str(&arg);
        ron_args.push(' ');
    }

    let ron_args_trimmed = ron_args.trim();

    let mut ron_args = String::with_capacity(
        ron_args_trimmed.len()
            + if ron_args_trimmed.starts_with('(') {
                2
            } else {
                0
            },
    );

    if !ron_args_trimmed.starts_with('(') {
        ron_args.push('(');
    }
    ron_args.push_str(ron_args_trimmed);
    if !ron_args_trimmed.starts_with('(') {
        ron_args.push(')');
    }

    ron_args
}

fn ron_options() -> Options {
    Options::default()
        .with_default_extension(Extensions::IMPLICIT_SOME)
        .with_default_extension(Extensions::UNWRAP_NEWTYPES)
        .with_default_extension(Extensions::UNWRAP_VARIANT_NEWTYPES)
}

#[allow(clippy::module_name_repetitions)]
pub fn try_parse<'de, D: Deserialize<'de>>(subcommand: &str, ron_args: &'de str) -> Result<D> {
    let mut de_ron = ron::Deserializer::from_str_with_options(ron_args, ron_options())
        .with_context(|| {
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
            let err = de_ron.error(err);

            Err(anyhow::anyhow!(
                "{}{}{}{} @ ({}):\n{}",
                subcommand,
                if path.iter().count() >= 1 { "." } else { "" },
                path,
                if path.iter().count() >= 1 { "" } else { "*" },
                err.position,
                err.code,
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
    let mut de_ron = ron::Deserializer::from_str_with_options(ron_args, ron_options())
        .with_context(|| {
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

pub fn try_print<S: Serialize>(value: &S) -> Result<String> {
    ron_options()
        .to_string_pretty(
            value,
            PrettyConfig::default()
                .decimal_floats(true)
                .struct_names(true),
        )
        .map_err(anyhow::Error::new)
}

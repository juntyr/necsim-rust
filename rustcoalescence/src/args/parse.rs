use anyhow::{Context, Result};
use ron::{extensions::Extensions, ser::PrettyConfig};
use serde::Deserialize;
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

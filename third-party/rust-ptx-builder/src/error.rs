use std::{fmt, path::PathBuf};

use colored::Colorize;
use semver::{Version, VersionReq};

#[macro_export]
macro_rules! bail {
    ($err:expr) => {
        return Err($err.into());
    };
}

pub type Error = anyhow::Error;
pub type Result<T> = anyhow::Result<T>;

#[derive(Debug, PartialEq, thiserror::Error, Clone)]
pub enum BuildErrorKind {
    CommandNotFound {
        command: String,
        hint: String,
    },

    CommandFailed {
        command: String,
        code: i32,
        stderr: String,
    },
    CommandVersionNotFulfilled {
        command: String,
        current: Version,
        required: VersionReq,
        hint: String,
    },

    InvalidCratePath(PathBuf),
    BuildFailed(Vec<String>),
    InvalidCrateType(String),
    MissingCrateType,
    InternalError(String),
    OtherError,
}

impl fmt::Display for BuildErrorKind {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        use BuildErrorKind::{
            BuildFailed, CommandFailed, CommandNotFound, CommandVersionNotFulfilled, InternalError,
            InvalidCratePath, InvalidCrateType, MissingCrateType, OtherError,
        };

        match self {
            CommandNotFound { command, hint } => write!(
                formatter,
                "Command not found in PATH: '{}'. {}.",
                command.bold(),
                hint.underline()
            ),

            CommandFailed {
                command,
                code,
                stderr,
            } => write!(
                formatter,
                "Command failed: '{}' with code '{}' and output:\n{}",
                command.bold(),
                code,
                stderr.trim(),
            ),

            CommandVersionNotFulfilled {
                command,
                current,
                required,
                hint,
            } => write!(
                formatter,
                "Command version is not fulfilled: '{}' is currently '{}' but '{}' is required. \
                 {}.",
                command.bold(),
                current.to_string().underline(),
                required.to_string().underline(),
                hint.underline(),
            ),

            InvalidCratePath(path) => write!(
                formatter,
                "{}: {}",
                "Invalid device crate path".bold(),
                path.display()
            ),

            BuildFailed(lines) => write!(
                formatter,
                "{}\n{}",
                "Unable to build a PTX crate!".bold(),
                lines.join("\n")
            ),

            InvalidCrateType(crate_type) => write!(
                formatter,
                "{}: the crate cannot be build as '{}'",
                "Impossible CrateType".bold(),
                crate_type
            ),

            MissingCrateType => write!(
                formatter,
                "{}: it's mandatory for mixed-type crates",
                "Missing CrateType".bold()
            ),

            InternalError(message) => write!(formatter, "{}: {}", "Internal error".bold(), message),
            OtherError => write!(formatter, "Other error"),
        }
    }
}

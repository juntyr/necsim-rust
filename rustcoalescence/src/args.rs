use std::{fmt, num::ParseIntError};

use derive_getters::Getters;
use structopt::StructOpt;

#[derive(Debug)]
#[non_exhaustive]
#[allow(clippy::empty_enum)]
pub enum Algorithm {
    #[cfg(feature = "necsim-classical")]
    Classical,
    #[cfg(feature = "necsim-gillespie")]
    Gillespie,
    #[cfg(feature = "necsim-skipping-gillespie")]
    SkippingGillespie,
    #[cfg(feature = "necsim-cuda")]
    CUDA,
}

impl std::fmt::Display for Algorithm {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::str::FromStr for Algorithm {
    type Err = String;

    fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
        #[allow(clippy::match_single_binding)]
        #[allow(clippy::wildcard_in_or_patterns)]
        match s {
            #[cfg(feature = "necsim-classical")]
            "Classical" | _ if s.eq_ignore_ascii_case("Classical") => Ok(Algorithm::Classical),
            #[cfg(feature = "necsim-gillespie")]
            "Gillespie" | _ if s.eq_ignore_ascii_case("Gillespie") => Ok(Algorithm::Gillespie),
            #[cfg(feature = "necsim-skipping-gillespie")]
            "SkippingGillespie" | _ if s.eq_ignore_ascii_case("SkippingGillespie") => {
                Ok(Algorithm::SkippingGillespie)
            },
            #[cfg(feature = "necsim-cuda")]
            "CUDA" | _ if s.eq_ignore_ascii_case("CUDA") => Ok(Algorithm::CUDA),
            _ => Err({
                let v: Vec<&'static str> = vec![
                    #[cfg(feature = "necsim-classical")]
                    "Classical",
                    #[cfg(feature = "necsim-gillespie")]
                    "Gillespie",
                    #[cfg(feature = "necsim-skipping-gillespie")]
                    "SkippingGillespie",
                    #[cfg(feature = "necsim-cuda")]
                    "CUDA",
                ];
                format!("valid values: {}", v.join(", "))
            }),
        }
    }
}

impl Algorithm {
    pub fn variants() -> Vec<&'static str> {
        vec![
            #[cfg(feature = "necsim-classical")]
            "Classical",
            #[cfg(feature = "necsim-gillespie")]
            "Gillespie",
            #[cfg(feature = "necsim-skipping-gillespie")]
            "SkippingGillespie",
            #[cfg(feature = "necsim-cuda")]
            "CUDA",
        ]
    }
}

#[derive(Debug, StructOpt, Getters)]
pub struct CommandLineArguments {
    #[structopt(flatten)]
    common_args: CommonArgs,
    #[structopt(subcommand)]
    command: Command,
}

#[derive(Debug, StructOpt, Getters)]
pub struct CommonArgs {
    #[structopt(long = "speciation")]
    speciation_probability_per_generation: f64,
    #[structopt(long = "sample")]
    sample_percentage: f64,
    #[structopt(long)]
    seed: u64,
    #[structopt(
        possible_values = &Algorithm::variants(),
        case_insensitive = true,
        long = "algorithm"
    )]
    algorithm: Algorithm,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    InMemory(InMemoryArgs),
    NonSpatial(NonSpatialArgs),
}

#[derive(Debug, StructOpt, Getters)]
pub struct InMemoryArgs {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
}

#[derive(Debug, StructOpt, Getters)]
pub struct NonSpatialArgs {
    #[structopt(parse(try_from_str = try_parse_area))]
    area: (u32, u32),
    deme: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseAreaError {
    TooManyDimensions,
    ParseIntError(ParseIntError),
}

impl fmt::Display for ParseAreaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyDimensions => "area can at most contain one '*'".fmt(f),
            Self::ParseIntError(error) => error.fmt(f),
        }
    }
}

impl From<ParseIntError> for ParseAreaError {
    fn from(error: ParseIntError) -> Self {
        Self::ParseIntError(error)
    }
}

fn try_parse_area(src: &str) -> Result<(u32, u32), ParseAreaError> {
    match src.find("*") {
        None => Ok((src.parse()?, 1)),
        Some(pos) => {
            if let Some(_) = src[(pos + 1)..].find("*") {
                return Err(ParseAreaError::TooManyDimensions);
            }

            let first = src[..pos].trim().parse()?;
            let second = src[(pos + 1)..].trim().parse()?;

            Ok((first, second))
        },
    }
}

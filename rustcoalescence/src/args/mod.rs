use derive_getters::Getters;
use structopt::StructOpt;

mod algorithm;
mod area;

pub use algorithm::Algorithm;
use area::try_parse_area;

#[derive(Debug, StructOpt, Getters)]
pub struct CommandLineArguments {
    #[structopt(subcommand)]
    command: Command,
    #[structopt(flatten)]
    common_args: CommonArgs,
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
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
#[allow(clippy::module_name_repetitions)]
pub struct InMemoryArgs {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
}

#[derive(Debug, StructOpt, Getters)]
#[allow(clippy::module_name_repetitions)]
pub struct NonSpatialArgs {
    #[structopt(parse(try_from_str = try_parse_area))]
    area: (u32, u32),
    deme: u32,
}

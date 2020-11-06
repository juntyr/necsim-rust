use derive_getters::Getters;
use structopt::clap::arg_enum;
use structopt::StructOpt;

arg_enum! {
    #[derive(Debug)]
    pub enum Algorithm {
        Classical,
        Gillespie,
        SkippingGillespie,
        CUDA,
    }
}

#[derive(Debug, StructOpt, Getters)]
pub struct CommandLineArguments {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf,
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf,
    #[structopt(long = "speciation")]
    speciation_probability_per_generation: f64,
    #[structopt(long = "sample")]
    sample_percentage: f64,
    #[structopt(long)]
    seed: u64,
    #[structopt(
        possible_values = &Algorithm::variants(),
        case_insensitive = true,
        default_value = "Classical",
        long = "algorithm"
    )]
    algorithm: Algorithm,
}

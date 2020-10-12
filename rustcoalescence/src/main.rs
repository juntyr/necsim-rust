use structopt::StructOpt;

/// Search for a pattern in a file and display the lines that contain it.
#[derive(Debug, StructOpt)]
struct Cli {
    #[structopt(parse(from_os_str))]
    habitat_map: std::path::PathBuf, // TODO: Check if exists
    #[structopt(parse(from_os_str))]
    dispersal_map: std::path::PathBuf, // TODO: Check if exists
    speciation_rate: f64,   // TODO: Check ]0; 1[
    sample_percentage: f64, // TODO: Check [0; 1]
    seed: u64,
}

fn main() {
    let args = Cli::from_args();

    println!("{:?}", args)
}

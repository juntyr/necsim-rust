use derive_getters::Getters;
use structopt::StructOpt;

#[derive(Debug)]
#[non_exhaustive]
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
        match s {
            #[cfg(feature = "necsim-classical")]
            "Classical" | _ if s.eq_ignore_ascii_case("Classical") => Ok(Algorithm::Classical),
            #[cfg(feature = "necsim-gillespie")]
            "Gillespie" | _ if s.eq_ignore_ascii_case("Gillespie") => Ok(Algorithm::Gillespie),
            #[cfg(feature = "necsim-skipping-gillespie")]
            "SkippingGillespie" | _ if s.eq_ignore_ascii_case("SkippingGillespie") => {
                Ok(Algorithm::SkippingGillespie)
            }
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
        long = "algorithm"
    )]
    algorithm: Algorithm,
}

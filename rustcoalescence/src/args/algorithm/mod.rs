use std::fmt;

#[cfg(feature = "necsim-cuda")]
pub mod cuda;

#[cfg(feature = "necsim-cuda")]
use cuda::CudaArguments;

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
    Cuda(CudaArguments),
    #[cfg(feature = "necsim-independent")]
    Independent,
}

impl fmt::Display for Algorithm {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
            "Skipping-Gillespie" | _ if s.eq_ignore_ascii_case("Skipping-Gillespie") => {
                Ok(Algorithm::SkippingGillespie)
            },
            #[cfg(feature = "necsim-cuda")]
            "Cuda" | _ if s.to_ascii_lowercase().starts_with("cuda") => {
                match s
                    .to_ascii_lowercase()
                    .strip_prefix("cuda")
                    .and_then(|s| s.strip_prefix("["))
                    .and_then(|s| s.strip_suffix("]"))
                {
                    Some(suffix) if !suffix.is_empty() => {
                        match ron::from_str(&format!("({})", suffix)) {
                            Ok(args) => Ok(Algorithm::Cuda(args)),
                            Err(err) => Err(format!(
                                "Invalid CUDA algorithm arguments [{}]: {}",
                                suffix, err
                            )),
                        }
                    },
                    _ => Ok(Algorithm::Cuda(CudaArguments::default())),
                }
            },
            #[cfg(feature = "necsim-independent")]
            "Independent" | _ if s.eq_ignore_ascii_case("Independent") => {
                Ok(Algorithm::Independent)
            },
            _ => Err(format!(
                "valid values: {}",
                Algorithm::variants().join(", ")
            )),
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
            "Skipping-Gillespie",
            #[cfg(feature = "necsim-cuda")]
            "CUDA",
            #[cfg(feature = "necsim-independent")]
            "Independent",
        ]
    }
}

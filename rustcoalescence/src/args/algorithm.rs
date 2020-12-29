use std::fmt;

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
            "CUDA" | _ if s.eq_ignore_ascii_case("CUDA") => Ok(Algorithm::CUDA),
            _ => Err({
                let v: Vec<&'static str> = vec![
                    #[cfg(feature = "necsim-classical")]
                    "Classical",
                    #[cfg(feature = "necsim-gillespie")]
                    "Gillespie",
                    #[cfg(feature = "necsim-skipping-gillespie")]
                    "Skipping-Gillespie",
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
            "Skipping-Gillespie",
            #[cfg(feature = "necsim-cuda")]
            "CUDA",
        ]
    }
}

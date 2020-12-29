use necsim_core::{
    cogs::{Habitat, SpeciationProbability},
    landscape::Location,
};

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[allow(clippy::module_name_repetitions)]
pub struct UniformSpeciationProbability {
    speciation_probability: f64,
}

impl UniformSpeciationProbability {
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&speciation_probability),
        "speciation_probability is a probability"
    )]
    pub fn new(speciation_probability: f64) -> Self {
        Self {
            speciation_probability,
        }
    }
}

#[contract_trait]
impl<H: Habitat> SpeciationProbability<H> for UniformSpeciationProbability {
    #[must_use]
    #[inline]
    fn get_speciation_probability_at_location(&self, _location: &Location) -> f64 {
        self.speciation_probability
    }
}

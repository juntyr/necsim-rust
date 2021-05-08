use necsim_core::{
    cogs::{Backup, Habitat, SpeciationProbability},
    landscape::Location,
};
use necsim_core_bond::ZeroInclOneInclF64;

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[allow(clippy::module_name_repetitions)]
pub struct UniformSpeciationProbability {
    speciation_probability: ZeroInclOneInclF64,
}

impl UniformSpeciationProbability {
    #[must_use]
    pub fn new(speciation_probability: ZeroInclOneInclF64) -> Self {
        Self {
            speciation_probability,
        }
    }
}

#[contract_trait]
impl Backup for UniformSpeciationProbability {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            speciation_probability: self.speciation_probability,
        }
    }
}

#[contract_trait]
impl<H: Habitat> SpeciationProbability<H> for UniformSpeciationProbability {
    #[must_use]
    #[inline]
    fn get_speciation_probability_at_location(
        &self,
        _location: &Location,
        _habitat: &H,
    ) -> ZeroInclOneInclF64 {
        self.speciation_probability
    }
}

use necsim_core::{
    cogs::{Backup, Habitat, SpeciationProbability},
    landscape::Location,
};
use necsim_core_bond::{ZeroExclOneInclF64, ZeroInclOneInclF64};

use crate::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat;

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitSpeciationProbability {
    meta_speciation_probability: ZeroExclOneInclF64,
}

impl SpatiallyImplicitSpeciationProbability {
    #[must_use]
    pub fn new(meta_speciation_probability: ZeroExclOneInclF64) -> Self {
        Self {
            meta_speciation_probability,
        }
    }
}

#[contract_trait]
impl Backup for SpatiallyImplicitSpeciationProbability {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            meta_speciation_probability: self.meta_speciation_probability,
        }
    }
}

#[contract_trait]
impl SpeciationProbability<SpatiallyImplicitHabitat> for SpatiallyImplicitSpeciationProbability {
    #[must_use]
    #[debug_requires(
        habitat.local().contains(location) || habitat.meta().contains(location),
        "location is inside either the local or meta habitat extent"
    )]
    #[inline]
    fn get_speciation_probability_at_location(
        &self,
        location: &Location,
        habitat: &SpatiallyImplicitHabitat,
    ) -> ZeroInclOneInclF64 {
        if habitat.local().contains(location) {
            ZeroInclOneInclF64::zero()
        } else {
            self.meta_speciation_probability.into()
        }
    }
}

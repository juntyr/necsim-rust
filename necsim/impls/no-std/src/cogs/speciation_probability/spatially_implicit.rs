use necsim_core::{
    cogs::{Backup, Habitat, SpeciationProbability},
    landscape::Location,
};
use necsim_core_bond::{ClosedUnitF64, PositiveUnitF64};

use crate::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat;

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCuda))]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitSpeciationProbability {
    meta_speciation_probability: PositiveUnitF64,
}

impl SpatiallyImplicitSpeciationProbability {
    #[must_use]
    pub fn new(meta_speciation_probability: PositiveUnitF64) -> Self {
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
    ) -> ClosedUnitF64 {
        if habitat.local().contains(location) {
            ClosedUnitF64::zero()
        } else {
            self.meta_speciation_probability.into()
        }
    }
}

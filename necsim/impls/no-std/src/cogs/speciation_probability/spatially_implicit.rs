use necsim_core::{
    cogs::{Habitat, SpeciationProbability},
    landscape::Location,
};

use crate::cogs::habitat::spatially_implicit::SpatiallyImplicitHabitat;

#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[allow(clippy::module_name_repetitions)]
pub struct SpatiallyImplicitSpeciationProbability {
    meta_speciation_probability: f64,
}

impl SpatiallyImplicitSpeciationProbability {
    #[debug_requires(
        (0.0_f64..=1.0_f64).contains(&meta_speciation_probability),
        "meta_speciation_probability is a probability"
    )]
    pub fn new(meta_speciation_probability: f64) -> Self {
        Self {
            meta_speciation_probability,
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
    ) -> f64 {
        if habitat.local().contains(location) {
            0.0_f64
        } else {
            self.meta_speciation_probability
        }
    }
}

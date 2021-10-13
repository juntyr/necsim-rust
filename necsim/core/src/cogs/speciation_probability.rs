use necsim_core_bond::ClosedUnitF64;

use crate::{
    cogs::{Habitat, MathsCore},
    landscape::Location,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait SpeciationProbability<M: MathsCore, H: Habitat<M>>:
    crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    fn get_speciation_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64;
}

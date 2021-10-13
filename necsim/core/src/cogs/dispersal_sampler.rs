use necsim_core_bond::ClosedUnitF64;

use crate::{
    cogs::{F64Core, RngCore},
    landscape::Location,
};

use super::Habitat;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait DispersalSampler<F: F64Core, H: Habitat<F>, G: RngCore<F>>:
    crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    #[debug_ensures(old(habitat).contains(&ret), "target is inside habitat")]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait SeparableDispersalSampler<F: F64Core, H: Habitat<F>, G: RngCore<F>>:
    DispersalSampler<F, H, G>
{
    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    #[debug_ensures(old(habitat).contains(&ret), "target is inside habitat")]
    #[debug_ensures(&ret != location, "disperses to a different location")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location;

    #[must_use]
    #[debug_requires(habitat.contains(location), "location is inside habitat")]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64;
}

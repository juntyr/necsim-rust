use necsim_core_bond::ClosedUnitF64;

use crate::{
    cogs::{MathsCore, Rng},
    landscape::Location,
};

use super::Habitat;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait DispersalSampler<M: MathsCore, H: Habitat<M>, G: Rng<M>>:
    crate::cogs::Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.is_location_habitable(location), "location is habitable")]
    #[debug_ensures(old(habitat).is_location_habitable(&ret), "target is habitable")]
    fn sample_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location;
}

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[allow(clippy::no_effect_underscore_binding)]
#[allow(clippy::module_name_repetitions)]
#[contract_trait]
pub trait SeparableDispersalSampler<M: MathsCore, H: Habitat<M>, G: Rng<M>>:
    DispersalSampler<M, H, G>
{
    #[must_use]
    #[debug_requires(habitat.is_location_habitable(location), "location is habitable")]
    #[debug_ensures(old(habitat).is_location_habitable(&ret), "target is habitable")]
    #[debug_ensures(&ret != location, "disperses to a different location")]
    fn sample_non_self_dispersal_from_location(
        &self,
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> Location;

    #[must_use]
    #[debug_requires(habitat.is_location_habitable(location), "location is habitable")]
    fn get_self_dispersal_probability_at_location(
        &self,
        location: &Location,
        habitat: &H,
    ) -> ClosedUnitF64;
}

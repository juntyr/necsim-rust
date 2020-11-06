use crate::landscape::Location;
use crate::rng::Rng;

use super::{Habitat, LineageReference, LineageStore};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>:
    core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(location) > 0, "location is habitable")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> Option<R>;
}

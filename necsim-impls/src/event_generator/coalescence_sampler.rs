use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use necsim_core::lineage::LineageReference;

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<L: LineageReference> {
    #[must_use]
    #[debug_requires(habitat > 0, "location is habitable")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: u32,
        rng: &mut impl Rng,
    ) -> Option<L>;
}

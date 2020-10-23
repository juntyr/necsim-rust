use necsim_corev2::cogs::{CoalescenceSampler, Habitat, LineageReference, LineageStore};
use necsim_corev2::landscape::Location;
use necsim_corev2::rng::Rng;

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalCoalescenceSampler;

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> CoalescenceSampler<H, R, S>
    for UnconditionalCoalescenceSampler
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut impl Rng,
    ) -> Option<R> {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            rng,
        )
    }
}

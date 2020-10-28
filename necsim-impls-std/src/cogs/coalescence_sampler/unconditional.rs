use necsim_core::cogs::{CoalescenceSampler, Habitat, LineageReference, LineageStore};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
pub struct UnconditionalCoalescenceSampler<
    H: Habitat,
    R: LineageReference<H>,
    S: LineageStore<H, R>,
>(std::marker::PhantomData<(H, R, S)>);

impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> Default
    for UnconditionalCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(std::marker::PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>> CoalescenceSampler<H, R, S>
    for UnconditionalCoalescenceSampler<H, R, S>
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

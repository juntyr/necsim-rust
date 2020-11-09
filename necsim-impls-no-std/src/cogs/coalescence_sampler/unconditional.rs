use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, CoherentLineageStore, Habitat, LineageReference, RngCore,
};
use necsim_core::landscape::Location;

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalCoalescenceSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(PhantomData<(H, G, R, S)>);

impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: CoherentLineageStore<H, R>> Default
    for UnconditionalCoalescenceSampler<H, G, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: CoherentLineageStore<H, R>>
    CoalescenceSampler<H, G, R, S> for UnconditionalCoalescenceSampler<H, G, R, S>
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: &Location,
        habitat: &H,
        lineage_store: &S,
        rng: &mut G,
    ) -> Option<R> {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            rng,
        )
    }
}

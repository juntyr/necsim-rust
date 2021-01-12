use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceRngSample, CoalescenceSampler, CoherentLineageStore, Habitat, LineageReference,
    },
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalCoalescenceSampler<
    H: Habitat,
    R: LineageReference<H>,
    S: CoherentLineageStore<H, R>,
>(PhantomData<(H, R, S)>);

impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>> Default
    for UnconditionalCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: CoherentLineageStore<H, R>> CoalescenceSampler<H, R, S>
    for UnconditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, Option<GlobalLineageReference>) {
        optional_coalescence::sample_optional_coalescence_at_location(
            location,
            habitat,
            lineage_store,
            coalescence_rng_sample,
        )
    }
}

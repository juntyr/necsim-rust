use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, Habitat,
        LineageReference, LocallyCoherentLineageStore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalCoalescenceSampler<
    H: Habitat,
    R: LineageReference<H>,
    S: LocallyCoherentLineageStore<H, R>,
>(PhantomData<(H, R, S)>);

impl<H: Habitat, R: LineageReference<H>, S: LocallyCoherentLineageStore<H, R>> Default
    for UnconditionalCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LocallyCoherentLineageStore<H, R>> Backup
    for UnconditionalCoalescenceSampler<H, R, S>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: LocallyCoherentLineageStore<H, R>>
    CoalescenceSampler<H, R, S> for UnconditionalCoalescenceSampler<H, R, S>
{
    #[must_use]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        optional_coalescence::sample_interaction_at_location(
            location,
            habitat,
            lineage_store,
            coalescence_rng_sample,
        )
    }
}

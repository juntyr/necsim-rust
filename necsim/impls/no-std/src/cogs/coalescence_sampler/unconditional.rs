use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, F64Core, Habitat,
        LineageReference, LocallyCoherentLineageStore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalCoalescenceSampler<
    F: F64Core,
    H: Habitat<F>,
    R: LineageReference<F, H>,
    S: LocallyCoherentLineageStore<F, H, R>,
>(PhantomData<(F, H, R, S)>);

impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
    > Default for UnconditionalCoalescenceSampler<F, H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(F, H, R, S)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
    > Backup for UnconditionalCoalescenceSampler<F, H, R, S>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(F, H, R, S)>)
    }
}

#[contract_trait]
impl<
        F: F64Core,
        H: Habitat<F>,
        R: LineageReference<F, H>,
        S: LocallyCoherentLineageStore<F, H, R>,
    > CoalescenceSampler<F, H, R, S> for UnconditionalCoalescenceSampler<F, H, R, S>
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

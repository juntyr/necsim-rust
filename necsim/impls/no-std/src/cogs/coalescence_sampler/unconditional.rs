use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, Habitat,
        LineageReference, LocallyCoherentLineageStore, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

use super::optional_coalescence;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct UnconditionalCoalescenceSampler<
    M: MathsCore,
    H: Habitat<M>,
    R: LineageReference<M, H>,
    S: LocallyCoherentLineageStore<M, H, R>,
>(PhantomData<(M, H, R, S)>);

impl<
        M: MathsCore,
        H: Habitat<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
    > Default for UnconditionalCoalescenceSampler<M, H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(M, H, R, S)>)
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
    > Backup for UnconditionalCoalescenceSampler<M, H, R, S>
{
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(M, H, R, S)>)
    }
}

#[contract_trait]
impl<
        M: MathsCore,
        H: Habitat<M>,
        R: LineageReference<M, H>,
        S: LocallyCoherentLineageStore<M, H, R>,
    > CoalescenceSampler<M, H, R, S> for UnconditionalCoalescenceSampler<M, H, R, S>
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

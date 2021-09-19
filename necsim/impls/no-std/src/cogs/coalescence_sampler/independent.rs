use core::marker::PhantomData;

use necsim_core::{
    cogs::{coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, Habitat},
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, LineageInteraction},
};

use crate::cogs::lineage_store::independent::IndependentLineageStore;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
pub struct IndependentCoalescenceSampler<H: Habitat>(PhantomData<H>);

impl<H: Habitat> Default for IndependentCoalescenceSampler<H> {
    fn default() -> Self {
        Self(PhantomData::<H>)
    }
}

#[contract_trait]
impl<H: Habitat> Backup for IndependentCoalescenceSampler<H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<H>)
    }
}

#[contract_trait]
impl<H: Habitat> CoalescenceSampler<H, GlobalLineageReference, IndependentLineageStore<H>>
    for IndependentCoalescenceSampler<H>
{
    #[must_use]
    #[debug_ensures(ret.1 == LineageInteraction::Maybe, "always reports maybe")]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        _lineage_store: &IndependentLineageStore<H>,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        let chosen_coalescence_index = coalescence_rng_sample
            .sample_coalescence_index(habitat.get_habitat_at_location(&location));

        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

        (indexed_location, LineageInteraction::Maybe)
    }
}

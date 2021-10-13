use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, F64Core, Habitat,
    },
    landscape::{IndexedLocation, Location},
    lineage::{GlobalLineageReference, LineageInteraction},
};

use crate::cogs::lineage_store::independent::IndependentLineageStore;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
pub struct IndependentCoalescenceSampler<F: F64Core, H: Habitat<F>>(PhantomData<(F, H)>);

impl<F: F64Core, H: Habitat<F>> Default for IndependentCoalescenceSampler<F, H> {
    fn default() -> Self {
        Self(PhantomData::<(F, H)>)
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>> Backup for IndependentCoalescenceSampler<F, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(F, H)>)
    }
}

#[contract_trait]
impl<F: F64Core, H: Habitat<F>>
    CoalescenceSampler<F, H, GlobalLineageReference, IndependentLineageStore<F, H>>
    for IndependentCoalescenceSampler<F, H>
{
    #[must_use]
    #[debug_ensures(ret.1 == LineageInteraction::Maybe, "always reports maybe")]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        _lineage_store: &IndependentLineageStore<F, H>,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        let chosen_coalescence_index = coalescence_rng_sample
            .sample_coalescence_index::<F>(habitat.get_habitat_at_location(&location));

        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

        (indexed_location, LineageInteraction::Maybe)
    }
}

use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::{
    cogs::{
        coalescence_sampler::CoalescenceRngSample, Backup, CoalescenceSampler, Habitat, MathsCore,
    },
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

use crate::cogs::lineage_store::{
    coherent::globally::singleton_demes::SingletonDemesHabitat,
    independent::IndependentLineageStore,
};

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
#[cfg_attr(feature = "cuda", cuda(free = "M", free = "H"))]
pub struct IndependentCoalescenceSampler<M: MathsCore, H: Habitat<M>>(PhantomData<(M, H)>);

impl<M: MathsCore, H: Habitat<M>> Default for IndependentCoalescenceSampler<M, H> {
    fn default() -> Self {
        Self(PhantomData::<(M, H)>)
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> Backup for IndependentCoalescenceSampler<M, H> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(PhantomData::<(M, H)>)
    }
}

#[contract_trait]
impl<M: MathsCore, H: Habitat<M>> CoalescenceSampler<M, H, IndependentLineageStore<M, H>>
    for IndependentCoalescenceSampler<M, H>
{
    #[must_use]
    // #[debug_ensures(ret.1 == LineageInteraction::Maybe, "always reports maybe")]
    default fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        _lineage_store: &IndependentLineageStore<M, H>,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        // Safety: individuals can only occupy habitable locations
        let habitat_at_location =
            unsafe { NonZeroU32::new_unchecked(habitat.get_habitat_at_location(&location)) };

        let chosen_coalescence_index =
            coalescence_rng_sample.sample_coalescence_index(habitat_at_location);

        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

        (indexed_location, LineageInteraction::Maybe)
    }
}

// Specialise for SingletonDemesHabitat as the compiler cannot yet optimise out
//  the call to `habitat.get_habitat_at_location(&location)`.
#[contract_trait]
impl<M: MathsCore, H: SingletonDemesHabitat<M>>
    CoalescenceSampler<M, H, IndependentLineageStore<M, H>>
    for IndependentCoalescenceSampler<M, H>
{
    #[must_use]
    #[debug_ensures(ret.1 == LineageInteraction::Maybe, "always reports maybe")]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        _habitat: &H,
        _lineage_store: &IndependentLineageStore<M, H>,
        _coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction) {
        (
            IndexedLocation::new(location, 0_u32),
            LineageInteraction::Maybe,
        )
    }
}

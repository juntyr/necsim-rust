use core::marker::PhantomData;

use necsim_core::{
    cogs::{
        CoalescenceRngSample, CoalescenceSampler, Habitat, IncoherentLineageStore, LineageReference,
    },
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentCoalescenceSampler<
    H: Habitat,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
>(PhantomData<(H, R, S)>);

impl<H: Habitat, R: LineageReference<H>, S: IncoherentLineageStore<H, R>> Default
    for IndependentCoalescenceSampler<H, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, R: LineageReference<H>, S: IncoherentLineageStore<H, R>>
    CoalescenceSampler<H, R, S> for IndependentCoalescenceSampler<H, R, S>
{
    #[must_use]
    #[debug_ensures(ret.1.is_none(), "never finds coalescence")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        _lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, Option<GlobalLineageReference>) {
        let chosen_coalescence_index = coalescence_rng_sample
            .sample_coalescence_index(habitat.get_habitat_at_location(&location));

        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index);

        (indexed_location, None)
    }
}

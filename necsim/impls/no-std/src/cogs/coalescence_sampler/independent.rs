use core::marker::PhantomData;

use necsim_core::{
    cogs::{CoalescenceSampler, Habitat, IncoherentLineageStore, LineageReference, RngCore},
    landscape::{IndexedLocation, Location},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(G: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rustacuda_core::DeviceCopy))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
#[derive(Debug)]
pub struct IndependentCoalescenceSampler<
    H: Habitat,
    G: RngCore,
    R: LineageReference<H>,
    S: IncoherentLineageStore<H, R>,
>(PhantomData<(H, G, R, S)>);

impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: IncoherentLineageStore<H, R>> Default
    for IndependentCoalescenceSampler<H, G, R, S>
{
    fn default() -> Self {
        Self(PhantomData::<(H, G, R, S)>)
    }
}

#[contract_trait]
impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: IncoherentLineageStore<H, R>>
    CoalescenceSampler<H, G, R, S> for IndependentCoalescenceSampler<H, G, R, S>
{
    #[must_use]
    #[debug_ensures(ret.1.is_none(), "never finds coalescence")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        _lineage_store: &S,
        rng: &mut G,
    ) -> (IndexedLocation, Option<R>) {
        use necsim_core::cogs::RngSampler;

        let chosen_coalescence_index =
            rng.sample_index(habitat.get_habitat_at_location(&location) as usize);

        #[allow(clippy::cast_possible_truncation)]
        let indexed_location = IndexedLocation::new(location, chosen_coalescence_index as u32);

        (indexed_location, None)
    }
}

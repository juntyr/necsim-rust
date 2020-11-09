use core::marker::PhantomData;

use necsim_core::cogs::{
    CoalescenceSampler, Habitat, IncoherentLineageStore, LineageReference, RngCore,
};
use necsim_core::landscape::Location;

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
    #[debug_ensures(ret.is_none(), "never finds coalescence")]
    fn sample_optional_coalescence_at_location(
        &self,
        _location: &Location,
        _habitat: &H,
        _lineage_store: &S,
        _rng: &mut G,
    ) -> Option<R> {
        None
    }
}

impl<H: Habitat, G: RngCore, R: LineageReference<H>, S: IncoherentLineageStore<H, R>>
    IndependentCoalescenceSampler<H, G, R, S>
{
    #[must_use]
    pub fn sample_coalescence_index_at_location(
        location: &Location,
        habitat: &H,
        rng: &mut G,
    ) -> usize {
        use necsim_core::cogs::RngSampler;

        rng.sample_index(habitat.get_habitat_at_location(location) as usize)
    }
}

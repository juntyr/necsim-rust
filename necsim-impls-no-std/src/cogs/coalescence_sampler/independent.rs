use core::marker::PhantomData;

use necsim_core::cogs::{CoalescenceSampler, Habitat, IncoherentLineageStore, LineageReference};
use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(H: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(R: rust_cuda::common::RustToCuda))]
#[cfg_attr(feature = "cuda", r2cBound(S: rust_cuda::common::RustToCuda))]
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
    #[debug_ensures(ret.is_none(), "never finds coalescence")]
    fn sample_optional_coalescence_at_location(
        &self,
        _location: &Location,
        _habitat: &H,
        _lineage_store: &S,
        _rng: &mut impl Rng,
    ) -> Option<R> {
        None
    }
}

impl<H: Habitat, R: LineageReference<H>, S: IncoherentLineageStore<H, R>>
    IndependentCoalescenceSampler<H, R, S>
{
    #[must_use]
    pub fn sample_coalescence_index_at_location(
        location: &Location,
        habitat: &H,
        rng: &mut impl Rng,
    ) -> usize {
        rng.sample_index(habitat.get_habitat_at_location(location) as usize)
    }
}

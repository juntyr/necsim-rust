use crate::{
    cogs::RngCore,
    landscape::{IndexedLocation, Location},
    lineage::GlobalLineageReference,
};

use super::{Habitat, LineageReference, LineageStore};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>:
    core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(&location) > 0, "location is habitable")]
    fn sample_optional_coalescence_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, Option<GlobalLineageReference>);
}

pub struct CoalescenceRngSample(f64);

impl CoalescenceRngSample {
    #[must_use]
    #[inline]
    pub fn new<G: RngCore>(rng: &mut G) -> Self {
        use crate::cogs::RngSampler;

        Self(rng.sample_uniform())
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length, "samples U(0, length - 1)")]
    pub fn sample_coalescence_index(self, length: u32) -> u32 {
        use crate::intrinsics::floor;

        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = floor(self.0 * f64::from(length)) as u32;
        index
    }
}

use core::{
    cmp::{Ord, Ordering},
    num::NonZeroU32,
};

use serde::{Deserialize, Serialize};

use crate::{
    cogs::{Backup, Habitat, LineageStore, MathsCore, Rng, RngCore},
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<M: MathsCore, H: Habitat<M>, S: LineageStore<M, H>>:
    Backup + core::fmt::Debug
{
    #[must_use]
    #[debug_requires(habitat.get_habitat_at_location(&location) > 0, "location is habitable")]
    fn sample_interaction_at_location(
        &self,
        location: Location,
        habitat: &H,
        lineage_store: &S,
        coalescence_rng_sample: CoalescenceRngSample,
    ) -> (IndexedLocation, LineageInteraction);
}

#[allow(clippy::unsafe_derive_deserialize)]
#[derive(Debug, PartialEq, Serialize, Deserialize, TypeLayout)]
#[repr(transparent)]
pub struct CoalescenceRngSample(u64);

#[contract_trait]
impl Backup for CoalescenceRngSample {
    unsafe fn backup_unchecked(&self) -> Self {
        Self(self.0)
    }
}

impl Ord for CoalescenceRngSample {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl PartialOrd for CoalescenceRngSample {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for CoalescenceRngSample {}

impl CoalescenceRngSample {
    #[must_use]
    #[inline]
    pub fn new<M: MathsCore, G: Rng<M>>(rng: &mut G) -> Self {
        Self(rng.generator().sample_u64())
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length.get(), "samples U(0, length - 1)")]
    pub fn sample_coalescence_index(self, length: NonZeroU32) -> u32 {
        // Sample U(0, length - 1) using a widening multiplication
        // Note: Some slight bias is traded for only needing one u64 sample
        // Note: Should optimise to a single 64 bit (high-only) multiplication
        #[allow(clippy::cast_possible_truncation)]
        {
            (((u128::from(self.0) * u128::from(length.get())) >> 64) & u128::from(!0_u32)) as u32
        }
    }
}

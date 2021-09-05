use core::cmp::{Ord, Ordering};

use serde::{Deserialize, Serialize};

use necsim_core_bond::ClosedUnitF64;

use crate::{
    cogs::{Backup, RngCore},
    landscape::{IndexedLocation, Location},
    lineage::LineageInteraction,
};

use super::{Habitat, LineageReference, LineageStore};

#[allow(clippy::inline_always, clippy::inline_fn_without_body)]
#[contract_trait]
pub trait CoalescenceSampler<H: Habitat, R: LineageReference<H>, S: LineageStore<H, R>>:
    crate::cogs::Backup + core::fmt::Debug
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

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct CoalescenceRngSample(ClosedUnitF64);

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
    pub fn new<G: RngCore>(rng: &mut G) -> Self {
        use crate::cogs::RngSampler;

        Self(rng.sample_uniform())
    }

    #[must_use]
    #[inline]
    #[debug_ensures(ret < length, "samples U(0, length - 1)")]
    pub fn sample_coalescence_index(self, length: u32) -> u32 {
        use necsim_core_f64::floor;

        // attributes on expressions are experimental
        // see https://github.com/rust-lang/rust/issues/15701
        #[allow(
            clippy::cast_precision_loss,
            clippy::cast_possible_truncation,
            clippy::cast_sign_loss
        )]
        let index = floor(self.0.get() * f64::from(length)) as u32;
        index
    }
}

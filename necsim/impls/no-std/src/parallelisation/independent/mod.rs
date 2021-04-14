use core::num::NonZeroUsize;

use serde::Deserialize;

use necsim_core_bond::PositiveF64;

use crate::cache::DirectMappedCache;

mod reporter;

pub mod individuals;
pub mod landscape;
pub mod monolithic;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AbsoluteDedupCache {
    pub capacity: NonZeroUsize,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RelativeDedupCache {
    pub factor: PositiveF64,
}

#[derive(Debug, Deserialize)]
pub enum DedupCache {
    Absolute(AbsoluteDedupCache),
    Relative(RelativeDedupCache),
    None,
}

impl DedupCache {
    #[must_use]
    pub fn construct<T: core::hash::Hash + PartialEq>(
        self,
        workload: usize,
    ) -> DirectMappedCache<T> {
        DirectMappedCache::with_capacity(match self {
            DedupCache::Absolute(AbsoluteDedupCache { capacity }) => capacity.get(),
            DedupCache::Relative(RelativeDedupCache { factor }) => {
                #[allow(
                    clippy::cast_precision_loss,
                    clippy::cast_sign_loss,
                    clippy::cast_possible_truncation
                )]
                let capacity = ((workload as f64) * factor.get()) as usize;

                capacity
            },
            DedupCache::None => 0_usize,
        })
    }
}

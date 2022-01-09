use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::OffByOneU32;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
pub struct AlmostInfiniteHabitat<M: MathsCore> {
    extent: LandscapeExtent,
    marker: PhantomData<M>,
}

impl<M: MathsCore> Default for AlmostInfiniteHabitat<M> {
    fn default() -> Self {
        Self {
            extent: LandscapeExtent::new(0_u32, 0_u32, OffByOneU32::max(), OffByOneU32::max()),
            marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Backup for AlmostInfiniteHabitat<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            extent: self.extent.clone(),
            marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for AlmostInfiniteHabitat<M> {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        // NOTE: Actually, u64::MAX + 1, does anyone care?
        u64::MAX
    }

    #[must_use]
    fn get_habitat_at_location(&self, _location: &Location) -> u32 {
        1_u32
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        (u64::from(indexed_location.location().y()) << 32)
            | u64::from(indexed_location.location().x())
    }
}

use core::marker::PhantomData;

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore, RngCore},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::{OffByOneU32, OffByOneU64};

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
    type LocationIterator<'a> = impl Iterator<Item = Location>;

    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> OffByOneU64 {
        OffByOneU64::max()
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

    #[must_use]
    fn sample_habitable_indexed_location<G: RngCore<M>>(&self, rng: &mut G) -> IndexedLocation {
        let index = rng.sample_u64();

        IndexedLocation::new(
            Location::new(
                (index & 0xFFFF_FFFF) as u32,
                ((index >> 32) & 0xFFFF_FFFF) as u32,
            ),
            0,
        )
    }

    #[must_use]
    fn iter_habitable_locations(&self) -> Self::LocationIterator<'_> {
        self.get_extent().iter()
    }
}

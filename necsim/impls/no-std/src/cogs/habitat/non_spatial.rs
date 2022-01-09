use core::{marker::PhantomData, num::NonZeroU32};

use necsim_core::{
    cogs::{Backup, Habitat, MathsCore},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};
use necsim_core_bond::OffByOneU32;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::LendRustToCuda))]
pub struct NonSpatialHabitat<M: MathsCore> {
    extent: LandscapeExtent,
    deme: NonZeroU32,
    marker: PhantomData<M>,
}

impl<M: MathsCore> NonSpatialHabitat<M> {
    #[must_use]
    #[debug_ensures(
        ret.get_total_habitat() == old(u64::from(area.0) * u64::from(area.1) * u64::from(deme.get())),
        "creates a habitat with community size area.0 * area.1 * deme"
    )]
    pub fn new(area: (OffByOneU32, OffByOneU32), deme: NonZeroU32) -> Self {
        Self {
            extent: LandscapeExtent::new(0, 0, area.0, area.1),
            deme,
            marker: PhantomData::<M>,
        }
    }

    pub(super) fn new_with_offset(
        x: u32,
        y: u32,
        area: (OffByOneU32, OffByOneU32),
        deme: NonZeroU32,
    ) -> Self {
        Self {
            extent: LandscapeExtent::new(x, y, area.0, area.1),
            deme,
            marker: PhantomData::<M>,
        }
    }

    #[must_use]
    pub fn get_deme(&self) -> NonZeroU32 {
        self.deme
    }
}

#[contract_trait]
impl<M: MathsCore> Backup for NonSpatialHabitat<M> {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            extent: self.extent.clone(),
            deme: self.deme,
            marker: PhantomData::<M>,
        }
    }
}

#[contract_trait]
impl<M: MathsCore> Habitat<M> for NonSpatialHabitat<M> {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        u64::from(self.extent.width())
            * u64::from(self.extent.height())
            * u64::from(self.deme.get())
    }

    #[must_use]
    fn get_habitat_at_location(&self, _location: &Location) -> u32 {
        self.deme.get()
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        u64::from(
            indexed_location
                .location()
                .y()
                .wrapping_sub(self.extent.y()),
        ) * u64::from(self.extent.width())
            + u64::from(
                indexed_location
                    .location()
                    .x()
                    .wrapping_sub(self.extent.x()),
            ) * u64::from(self.deme.get())
            + u64::from(indexed_location.index())
    }
}

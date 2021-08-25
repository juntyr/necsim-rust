use necsim_core::{
    cogs::{Backup, Habitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

use crate::cogs::habitat::non_spatial::NonSpatialHabitat;

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
pub struct SpatiallyImplicitHabitat {
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    local: NonSpatialHabitat,
    #[cfg_attr(feature = "cuda", r2cEmbed)]
    meta: NonSpatialHabitat,
    extent: LandscapeExtent,
}

impl SpatiallyImplicitHabitat {
    #[must_use]
    #[debug_ensures(
        ret.get_total_habitat() == old(
            u64::from(local_area.0) * u64::from(local_area.1) * u64::from(local_deme)
            + u64::from(meta_area.0) * u64::from(meta_area.1) * u64::from(meta_deme)
        ),
        "creates a habitat with a combined local and meta community size "
    )]
    pub fn new(
        local_area: (u32, u32),
        local_deme: u32,
        meta_area: (u32, u32),
        meta_deme: u32,
    ) -> Self {
        let local = NonSpatialHabitat::new(local_area, local_deme);
        let meta = NonSpatialHabitat::new_with_offset(
            local.get_extent().width(),
            local.get_extent().height(),
            meta_area,
            meta_deme,
        );

        Self {
            extent: LandscapeExtent::new(
                0,
                0,
                local.get_extent().width() + meta.get_extent().width(),
                local.get_extent().height() + meta.get_extent().height(),
            ),
            local,
            meta,
        }
    }

    #[must_use]
    pub fn local(&self) -> &NonSpatialHabitat {
        &self.local
    }

    #[must_use]
    pub fn meta(&self) -> &NonSpatialHabitat {
        &self.meta
    }
}

#[contract_trait]
impl Backup for SpatiallyImplicitHabitat {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            local: self.local.backup_unchecked(),
            meta: self.meta.backup_unchecked(),
            extent: self.extent.clone(),
        }
    }
}

#[contract_trait]
impl Habitat for SpatiallyImplicitHabitat {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        self.local.get_total_habitat() + self.meta().get_total_habitat()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        if self.local.get_extent().contains(location) {
            self.local.get_habitat_at_location(location)
        } else if self.meta().get_extent().contains(location) {
            self.meta.get_habitat_at_location(location)
        } else {
            0_u32
        }
    }

    #[must_use]
    #[inline]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        if self
            .local
            .get_extent()
            .contains(indexed_location.location())
        {
            self.local
                .map_indexed_location_to_u64_injective(indexed_location)
        } else {
            self.meta
                .map_indexed_location_to_u64_injective(indexed_location)
        }
    }
}

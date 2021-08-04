use necsim_core::{
    cogs::{Backup, Habitat},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(rust_cuda::common::RustToCudaAsRust))]
#[derive(Debug)]
pub struct AlmostInfiniteHabitat {
    extent: LandscapeExtent,
}

impl Default for AlmostInfiniteHabitat {
    fn default() -> Self {
        Self {
            extent: LandscapeExtent::new(0_u32, 0_u32, u32::MAX, u32::MAX),
        }
    }
}

#[contract_trait]
impl Backup for AlmostInfiniteHabitat {
    unsafe fn backup_unchecked(&self) -> Self {
        Self {
            extent: self.extent.clone(),
        }
    }
}

#[contract_trait]
impl Habitat for AlmostInfiniteHabitat {
    #[must_use]
    fn get_extent(&self) -> &LandscapeExtent {
        &self.extent
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        u64::from(u32::MAX) * u64::from(u32::MAX)
    }

    #[must_use]
    fn get_habitat_at_location(&self, _location: &Location) -> u32 {
        1_u32
    }

    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        u64::from(indexed_location.location().y()) * u64::from(u32::MAX)
            + u64::from(indexed_location.location().x())
    }
}

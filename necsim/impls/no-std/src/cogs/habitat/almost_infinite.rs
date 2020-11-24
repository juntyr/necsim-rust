use necsim_core::{
    cogs::{Habitat, HabitatToU64Injection},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct AlmostInfiniteHabitat(());

impl Default for AlmostInfiniteHabitat {
    fn default() -> Self {
        Self(())
    }
}

#[contract_trait]
impl Habitat for AlmostInfiniteHabitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        LandscapeExtent::new(0_u32, 0_u32, u32::MAX, u32::MAX)
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        u64::from(u32::MAX) * u64::from(u32::MAX)
    }

    #[must_use]
    fn get_habitat_at_location(&self, _location: &Location) -> u32 {
        1_u32
    }
}

#[contract_trait]
impl HabitatToU64Injection for AlmostInfiniteHabitat {
    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        u64::from(indexed_location.location().y()) * u64::from(u32::MAX)
            + u64::from(indexed_location.location().x())
    }
}

use necsim_core::{
    cogs::{Habitat, HabitatToU64Injection},
    landscape::{IndexedLocation, LandscapeExtent, Location},
};

#[allow(clippy::module_name_repetitions)]
#[cfg_attr(feature = "cuda", derive(RustToCuda))]
#[derive(Debug)]
pub struct NonSpatialHabitat {
    extent: LandscapeExtent,
    deme: u32,
}

impl NonSpatialHabitat {
    #[must_use]
    #[debug_ensures(
        ret.get_total_habitat() == old(u64::from(area.0) * u64::from(area.1) * u64::from(deme)),
        "creates a habitat with community size area.0 * area.1 * deme"
    )]
    pub fn new(area: (u32, u32), deme: u32) -> Self {
        Self {
            extent: LandscapeExtent::new(0, 0, area.0, area.1),
            deme,
        }
    }
}

#[contract_trait]
impl Habitat for NonSpatialHabitat {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        self.extent.clone()
    }

    #[must_use]
    fn get_total_habitat(&self) -> u64 {
        u64::from(self.extent.width()) * u64::from(self.extent.height()) * u64::from(self.deme)
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        if self.extent.contains(location) {
            self.deme
        } else {
            0_u32
        }
    }
}

#[contract_trait]
impl HabitatToU64Injection for NonSpatialHabitat {
    #[must_use]
    fn map_indexed_location_to_u64_injective(&self, indexed_location: &IndexedLocation) -> u64 {
        (u64::from(indexed_location.location().y()) * u64::from(self.extent.width())
            + u64::from(indexed_location.location().x()))
            * u64::from(self.deme)
            + u64::from(indexed_location.index())
    }
}

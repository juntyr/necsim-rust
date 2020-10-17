use array2d::Array2D;

use necsim_core::landscape::{Landscape, LandscapeExtent, Location};
use necsim_core::rng::Rng;

use super::dispersal::Dispersal;
use super::habitat::Habitat;

use super::dispersal::in_memory_precalculated::{
    InMemoryPrecalculatedDispersal, InconsistentDispersalMapSize,
};
use super::habitat::in_memory::InMemoryHabitat;

#[allow(clippy::module_name_repetitions)]
pub struct LandscapeInMemoryHabitatInMemoryPrecalculatedDispersal {
    habitat: InMemoryHabitat,
    dispersal: InMemoryPrecalculatedDispersal,
}

#[contract_trait]
impl Landscape for LandscapeInMemoryHabitatInMemoryPrecalculatedDispersal {
    #[must_use]
    fn get_extent(&self) -> LandscapeExtent {
        self.habitat.get_extent()
    }

    #[must_use]
    fn get_total_habitat(&self) -> usize {
        self.habitat.get_total_habitat()
    }

    #[must_use]
    fn get_habitat_at_location(&self, location: &Location) -> u32 {
        self.habitat.get_habitat_at_location(location)
    }

    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location {
        self.dispersal.sample_dispersal_from_location(location, rng)
    }
}

impl LandscapeInMemoryHabitatInMemoryPrecalculatedDispersal {
    /// Creates a new `LandscapeInMemoryWithPrecalculatedDispersal` from the
    /// `habitat` and `dispersal` map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    #[debug_ensures(
        ret.is_ok() == (
            dispersal.num_columns() == old(habitat.num_elements()) &&
            dispersal.num_rows() == old(habitat.num_elements())
        ), "returns error iff dispersal dimensions inconsistent"
    )]
    pub fn new(
        habitat: Array2D<u32>,
        dispersal: &Array2D<f64>,
    ) -> Result<Self, InconsistentDispersalMapSize> {
        let habitat = InMemoryHabitat::new(habitat);
        let dispersal = InMemoryPrecalculatedDispersal::new(dispersal, habitat.get_extent())?;

        Ok(Self { habitat, dispersal })
    }
}

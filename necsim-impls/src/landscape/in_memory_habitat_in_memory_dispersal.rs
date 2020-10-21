use array2d::Array2D;

use necsim_core::landscape::{Landscape, LandscapeExtent, Location};
use necsim_core::rng::Rng;

use crate::landscape::dispersal::in_memory::InMemoryDispersal;
use crate::landscape::habitat::in_memory::InMemoryHabitat;
use crate::landscape::habitat::Habitat;

use super::dispersal::in_memory::error::InMemoryDispersalError;

#[allow(clippy::module_name_repetitions)]
pub struct LandscapeInMemoryHabitatInMemoryDispersal<D: InMemoryDispersal> {
    habitat: InMemoryHabitat,
    dispersal: D,
}

#[contract_trait]
impl<D: InMemoryDispersal> Landscape for LandscapeInMemoryHabitatInMemoryDispersal<D> {
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

impl<D: InMemoryDispersal> LandscapeInMemoryHabitatInMemoryDispersal<D> {
    /// Creates a new `LandscapeInMemoryHabitatInMemoryDispersal` from the
    /// `habitat` and `dispersal` map.
    ///
    /// # Errors
    ///
    /// `Err(InconsistentDispersalMapSize)` is returned iff the dimensions of
    /// `dispersal` are not `ExE` given `E=RxC` where `habitat` has dimension
    /// `RxC`.
    ///
    /// `Err(InconsistentDispersalProbabilities)` is returned iff any of the
    /// following conditions is violated:
    /// - habitat cells must disperse somewhere
    /// - non-habitat cells must not disperse
    /// - dispersal must only target habitat cells
    pub fn new(
        habitat: Array2D<u32>,
        dispersal: &Array2D<f64>,
    ) -> Result<Self, InMemoryDispersalError> {
        let habitat = InMemoryHabitat::new(habitat);
        let dispersal = D::new(dispersal, &habitat)?;

        Ok(Self { habitat, dispersal })
    }
}

use necsim_core::landscape::Location;
use necsim_core::rng::Rng;

pub mod in_memory;

pub trait Dispersal {
    #[must_use]
    fn sample_dispersal_from_location(&self, location: &Location, rng: &mut impl Rng) -> Location;
}
